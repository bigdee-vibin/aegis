"""Shared helpers for the Aegis synthetic corpora generator (story S0.4). Stdlib only.

Everything produced with these helpers is synthetic: random characters, invented
names, reserved/fictional ranges, or publicly documented test values.
"""
import base64
import json
import random

VERSION = "1.0.0"
SEED_PREFIX = "aegis-s0.4-corpora-v1"

CATEGORIES = {"identity", "credential_secret", "financial", "confidential", "technical",
              "personal_record", "contextual_identity", "defence", "prompt_injection"}
SEVERITIES = {"info", "low", "medium", "high", "critical"}
ACTIONS = {"allow", "notify", "review", "block"}
_SEV_ACTION = {"info": "allow", "low": "notify", "medium": "review", "high": "review", "critical": "block"}
_ORDER = ["allow", "notify", "review", "block"]


def rng_for(name):
    # str seeds are hashed with SHA-512 by random.seed(): stable across runs/platforms.
    return random.Random(f"{SEED_PREFIX}:{name}")


UP = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
LOW = "abcdefghijklmnopqrstuvwxyz"
DIG = "0123456789"
ALNUM = UP + LOW + DIG
B64URL = ALNUM + "-_"
B64STD = ALNUM + "+/"
HEXL = "0123456789abcdef"
UPDIG = UP + DIG


def rs(r, n, alpha=ALNUM):
    return "".join(r.choice(alpha) for _ in range(n))


def b64u(b):
    return base64.urlsafe_b64encode(b).decode().rstrip("=")


class B:
    """Builds text while tracking labelled spans (UTF-8 byte offsets, end exclusive)."""

    def __init__(self):
        self.s = ""
        self.labels = []
        self.rw = ""      # sanitised reference rewrite (preservation corpus)
        self.pres = []    # must-preserve items (preservation corpus)

    def t(self, txt):
        self.s += txt
        self.rw += txt
        return self

    def L(self, value, category, subtype, severity, placeholder=None, **extra):
        cs = len(self.s)
        bs = len(self.s.encode("utf-8"))
        self.s += value
        lab = {"category": category, "subtype": subtype, "start": bs,
               "end": bs + len(value.encode("utf-8")), "severity": severity,
               "char_start": cs, "char_end": cs + len(value)}
        lab.update(extra)
        self.labels.append(lab)
        self.rw += placeholder if placeholder is not None else f"[{subtype.upper()}]"
        return self

    def K(self, value, kind):
        bs = len(self.s.encode("utf-8"))
        self.s += value
        self.rw += value
        self.pres.append({"kind": kind, "text": value, "start": bs, "end": bs + len(value.encode("utf-8"))})
        return self


def action_for(labels, default="allow"):
    best = default
    for lab in labels:
        a = _SEV_ACTION[lab["severity"]]
        if _ORDER.index(a) > _ORDER.index(best):
            best = a
    return best


class Corpus:
    def __init__(self, name, prefix):
        self.name, self.prefix, self.rows = name, prefix, []
        self.r = rng_for(name)

    def add(self, text, labels, action=None, notes="", **extra):
        rid = f"{self.prefix}-{len(self.rows) + 1:04d}"
        if action is None:
            action = action_for(labels)
        row = {"id": rid, "corpus_version": VERSION, "corpus": self.name, "text": text, "labels": labels,
               "expected_action": action, "notes": notes}
        row.update(extra)
        self.rows.append(row)
        return row

    def addb(self, b, action=None, notes="", **extra):
        return self.add(b.s, b.labels, action, notes, **extra)


# ------------------------------------------------------------------ checksums
def luhn_check_digit(payload):
    total = 0
    for i, ch in enumerate(reversed(payload)):
        d = int(ch)
        if i % 2 == 0:
            d *= 2
            if d > 9:
                d -= 9
        total += d
    return str((10 - total % 10) % 10)


def luhn_ok(num):
    return luhn_check_digit(num[:-1]) == num[-1]


def iban_check(country, bban):
    n = int("".join(str(int(c, 36)) for c in bban + country + "00"))
    return f"{98 - n % 97:02d}"


def iban_ok(iban):
    s = iban[4:] + iban[:4]
    return int("".join(str(int(c, 36)) for c in s)) % 97 == 1


_VD = [[0, 1, 2, 3, 4, 5, 6, 7, 8, 9], [1, 2, 3, 4, 0, 6, 7, 8, 9, 5], [2, 3, 4, 0, 1, 7, 8, 9, 5, 6],
       [3, 4, 0, 1, 2, 8, 9, 5, 6, 7], [4, 0, 1, 2, 3, 9, 5, 6, 7, 8], [5, 9, 8, 7, 6, 0, 4, 3, 2, 1],
       [6, 5, 9, 8, 7, 1, 0, 4, 3, 2], [7, 6, 5, 9, 8, 2, 1, 0, 4, 3], [8, 7, 6, 5, 9, 3, 2, 1, 0, 4],
       [9, 8, 7, 6, 5, 4, 3, 2, 1, 0]]
_VP = [[0, 1, 2, 3, 4, 5, 6, 7, 8, 9], [1, 5, 7, 6, 2, 8, 3, 0, 9, 4], [5, 8, 0, 3, 7, 9, 6, 1, 4, 2],
       [8, 9, 1, 6, 0, 4, 3, 5, 2, 7], [9, 4, 5, 3, 1, 2, 6, 8, 7, 0], [4, 2, 8, 6, 5, 7, 3, 9, 0, 1],
       [2, 7, 9, 3, 8, 0, 6, 4, 1, 5], [7, 0, 4, 6, 9, 1, 3, 2, 5, 8]]
_VINV = [0, 4, 3, 2, 1, 5, 6, 7, 8, 9]


def verhoeff_digit(payload):
    c = 0
    for i, ch in enumerate(reversed(payload)):
        c = _VD[c][_VP[(i + 1) % 8][int(ch)]]
    return str(_VINV[c])


def verhoeff_ok(num):
    c = 0
    for i, ch in enumerate(reversed(num)):
        c = _VD[c][_VP[i % 8][int(ch)]]
    return c == 0


def cpf_digits(nine):
    d = [int(x) for x in nine]
    for n in (9, 10):
        s = sum(d[i] * (n + 1 - i) for i in range(n))
        d.append((s * 10 % 11) % 10)
    return "".join(map(str, d))


def nric_letter(digits7, prefix="S"):
    w = [2, 7, 6, 5, 4, 3, 2]
    s = sum(int(a) * b for a, b in zip(digits7, w)) + (4 if prefix == "T" else 0)
    return "JZIHGFEDCBA"[s % 11]


def bsn_ok(n9):
    w = [9, 8, 7, 6, 5, 4, 3, 2, -1]
    return sum(int(a) * b for a, b in zip(n9, w)) % 11 == 0


def dni_letter(n):
    return "TRWAGMYFPDXBNJZSQVHLCKE"[n % 23]


# --------------------------------------------------------------- invented data
FIRST_A = ["Mar", "Tel", "Bren", "Ka", "Lio", "Ves", "Dor", "Ila", "Nor", "Sev", "Yan", "Tor", "Eli", "Quin"]
FIRST_B = ["velin", "dra", "wen", "rik", "lara", "ton", "mira", "sel", "dar", "nova", "sha", "bert"]
LAST_A = ["Hark", "Tarv", "Ost", "Fenn", "Brask", "Quel", "Mord", "Vint", "Ald", "Pell", "Strom", "Wex"]
LAST_B = ["wick", "ndell", "brook", "ley", "orth", "ssen", "mere", "field", "stad", "croft", "ova", "lund"]
STREET_A = ["Alder", "Brindle", "Cobble", "Dunmore", "Ember", "Fallow", "Gorse", "Hollin", "Ivel", "Juniper", "Kestrel"]
STREET_T = ["Lane", "Road", "Street", "Close", "Way", "Avenue", "Terrace"]
TOWNS = [("Brindlemouth", "United Kingdom"), ("Port Vellacott", "Ireland"), ("Hallowmere", "Canada"),
         ("Kestrel Bay", "Australia"), ("Northwick-on-Tarn", "United Kingdom"), ("Ostenfield", "Germany"),
         ("Valdemar Heights", "Sweden"), ("San Ormeno", "Spain"), ("Lac-Tessier", "France")]
DOMAINS = ["example.com", "example.org", "example.net", "mail.example.com", "corp.example.org", "inbox.example.test"]


def person(r):
    return r.choice(FIRST_A) + r.choice(FIRST_B), r.choice(LAST_A) + r.choice(LAST_B)


def email_for(r, first, last):
    f, l = first.lower(), last.lower()
    local = [f"{f}.{l}", f"{f}_{l}", f"{f[0]}{l}", f"{f}.{l}+{rs(r, 4, LOW)}", f"{l}{r.randrange(10, 99)}",
             f"{f}-{l}"][r.randrange(6)]
    return f"{local}@{r.choice(DOMAINS)}"


def address(r):
    town, country = r.choice(TOWNS)
    pc = r.choice([f"{rs(r, 2, UP)}{r.randrange(1, 9)} {r.randrange(1, 9)}{rs(r, 2, UP)}",
                   f"{r.randrange(10000, 99999)}",
                   f"{rs(r, 1, UP)}{r.randrange(1, 9)}{rs(r, 1, UP)} {r.randrange(1, 9)}{rs(r, 1, UP)}{r.randrange(1, 9)}"])
    return f"{r.randrange(1, 240)} {r.choice(STREET_A)} {r.choice(STREET_T)}, {town}, {pc}, {country}"


def fake_pem(r, ptype, lines):
    raw = b"AEGIS-SYNTHETIC-NOT-A-KEY:" + r.randbytes(48 * lines)
    body = base64.b64encode(raw).decode()
    chunks = [body[i:i + 64] for i in range(0, len(body), 64)][:lines]
    return f"-----BEGIN {ptype}-----\n" + "\n".join(chunks) + f"\n-----END {ptype}-----"


def fake_jwt(r, alg="HS256"):
    h = b64u(json.dumps({"alg": alg, "typ": "JWT"}, separators=(",", ":")).encode())
    p = b64u(json.dumps({"sub": f"synthetic-user-{r.randrange(1000, 9999)}", "iss": "https://issuer.example.test",
                         "iat": 1760000000 + r.randrange(10 ** 6), "scope": "read"}, separators=(",", ":")).encode())
    return f"{h}.{p}.{rs(r, 43, B64URL)}"


def provider_specs():
    """(subtype, display, generator(r)->value, env_name, top30, context_bound)."""
    return [
        ("anthropic_key", "Anthropic", lambda r: "sk-ant-api03-" + rs(r, 93, B64URL) + "AA", "ANTHROPIC_API_KEY", True, False),
        ("openai_key", "OpenAI project", lambda r: "sk-proj-" + rs(r, 64, B64URL), "OPENAI_API_KEY", True, False),
        ("openai_key", "OpenAI legacy", lambda r: "sk-" + rs(r, 20) + "T3BlbkFJ" + rs(r, 20), "OPENAI_API_KEY", True, False),
        ("aws_access_key", "AWS access key id", lambda r: "AKIA" + rs(r, 16, UPDIG), "AWS_ACCESS_KEY_ID", True, False),
        ("aws_secret_key", "AWS secret access key", lambda r: rs(r, 40, B64STD), "AWS_SECRET_ACCESS_KEY", True, True),
        ("github_token", "GitHub PAT classic", lambda r: "ghp_" + rs(r, 36), "GITHUB_TOKEN", True, False),
        ("github_token", "GitHub fine-grained PAT", lambda r: "github_pat_" + rs(r, 22) + "_" + rs(r, 59), "GH_TOKEN", True, False),
        ("github_token", "GitHub OAuth token", lambda r: "gho_" + rs(r, 36), "GITHUB_OAUTH_TOKEN", True, False),
        ("gitlab_token", "GitLab PAT", lambda r: "glpat-" + rs(r, 20, B64URL), "GITLAB_TOKEN", True, False),
        ("slack_token", "Slack bot token", lambda r: f"xoxb-{rs(r, 12, DIG)}-{rs(r, 13, DIG)}-{rs(r, 24)}", "SLACK_BOT_TOKEN", True, False),
        ("slack_webhook", "Slack webhook URL", lambda r: f"https://hooks.slack.com/services/T{rs(r, 9, UPDIG)}/B{rs(r, 9, UPDIG)}/{rs(r, 24)}", "SLACK_WEBHOOK_URL", True, False),
        ("google_api_key", "Google API key", lambda r: "AIza" + rs(r, 35, B64URL), "GOOGLE_API_KEY", True, False),
        ("google_oauth_secret", "Google OAuth client secret", lambda r: "GOCSPX-" + rs(r, 28, B64URL), "GOOGLE_CLIENT_SECRET", True, False),
        ("stripe_live_key", "Stripe live secret", lambda r: "sk_live_" + rs(r, 24), "STRIPE_SECRET_KEY", True, False),
        ("stripe_live_key", "Stripe live restricted", lambda r: "rk_live_" + rs(r, 24), "STRIPE_RESTRICTED_KEY", True, False),
        ("stripe_webhook_secret", "Stripe webhook secret", lambda r: "whsec_" + rs(r, 32), "STRIPE_WEBHOOK_SECRET", True, False),
        ("huggingface_token", "Hugging Face", lambda r: "hf_" + rs(r, 34), "HF_TOKEN", True, False),
        ("npm_token", "npm", lambda r: "npm_" + rs(r, 36), "NPM_TOKEN", True, False),
        ("sendgrid_key", "SendGrid", lambda r: f"SG.{rs(r, 22, B64URL)}.{rs(r, 43, B64URL)}", "SENDGRID_API_KEY", True, False),
        ("twilio_api_key", "Twilio API key SID", lambda r: "SK" + rs(r, 32, HEXL), "TWILIO_API_KEY", True, False),
        ("mailgun_key", "Mailgun", lambda r: "key-" + rs(r, 32, HEXL), "MAILGUN_API_KEY", True, False),
        ("mailchimp_key", "Mailchimp", lambda r: rs(r, 32, HEXL) + f"-us{r.randrange(1, 22)}", "MAILCHIMP_API_KEY", True, False),
        ("digitalocean_token", "DigitalOcean", lambda r: "dop_v1_" + rs(r, 64, HEXL), "DIGITALOCEAN_TOKEN", True, False),
        ("shopify_token", "Shopify admin", lambda r: "shpat_" + rs(r, 32, HEXL), "SHOPIFY_ACCESS_TOKEN", True, False),
        ("square_token", "Square", lambda r: "EAAA" + rs(r, 60, B64URL), "SQUARE_ACCESS_TOKEN", True, False),
        ("pypi_token", "PyPI", lambda r: "pypi-AgEIcHlwaS5vcmc" + rs(r, 70, B64URL), "PYPI_TOKEN", True, False),
        ("azure_storage_key", "Azure storage account key", lambda r: rs(r, 86, B64STD) + "==", "AZURE_STORAGE_KEY", True, True),
        ("discord_token", "Discord bot", lambda r: r.choice("MNO") + rs(r, 23, B64URL) + "." + rs(r, 6, B64URL) + "." + rs(r, 27, B64URL), "DISCORD_BOT_TOKEN", True, False),
        ("telegram_bot_token", "Telegram bot", lambda r: rs(r, 10, DIG) + ":AA" + rs(r, 33, B64URL), "TELEGRAM_BOT_TOKEN", True, False),
        ("notion_token", "Notion integration", lambda r: "secret_" + rs(r, 43), "NOTION_TOKEN", True, False),
        # extras beyond the top 30
        ("databricks_token", "Databricks", lambda r: "dapi" + rs(r, 32, HEXL), "DATABRICKS_TOKEN", False, False),
        ("linear_key", "Linear", lambda r: "lin_api_" + rs(r, 40), "LINEAR_API_KEY", False, False),
        ("postman_key", "Postman", lambda r: f"PMAK-{rs(r, 24, HEXL)}-{rs(r, 34, HEXL)}", "POSTMAN_API_KEY", False, False),
        ("groq_key", "Groq", lambda r: "gsk_" + rs(r, 52), "GROQ_API_KEY", False, False),
        ("replicate_token", "Replicate", lambda r: "r8_" + rs(r, 37), "REPLICATE_API_TOKEN", False, False),
        ("cloudflare_token", "Cloudflare API token", lambda r: rs(r, 40, B64URL), "CLOUDFLARE_API_TOKEN", False, True),
        ("doppler_token", "Doppler", lambda r: "dp.st.dev." + rs(r, 44), "DOPPLER_TOKEN", False, False),
        ("openrouter_key", "OpenRouter", lambda r: "sk-or-v1-" + rs(r, 64, HEXL), "OPENROUTER_API_KEY", False, False),
    ]
