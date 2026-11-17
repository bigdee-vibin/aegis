"""Secrets corpora: positives and hard negatives (S0.4 part a)."""
import base64
import hashlib

from cgen_common import (B, Corpus, B64STD, B64URL, ALNUM, DIG, HEXL, LOW, UPDIG, UP, address, fake_jwt, fake_pem,
                         provider_specs, rs)

CTXS = [  # (name, has_name, template). {S}=secret {N}=ENV_NAME {n}=json_name
    ("prompt", False, "Can you help me debug this? I set the key to {S} but the API still returns 401. What am I doing wrong?"),
    ("env", True, "# .env.local\nAPP_ENV=production\n{N}={S}\nLOG_LEVEL=info\n"),
    ("python", True, "import os\nclient = Client(api_key=\"{S}\")  # {N}\nprint(client.ping())\n"),
    ("json", True, '{"service": "billing", "{n}": "{S}", "retries": 3}'),
    ("curl", False, 'curl -s -H "Authorization: Bearer {S}" https://api.example.test/v1/items'),
    ("yaml", True, "services:\n  worker:\n    environment:\n      - {N}={S}\n"),
    ("log", False, "2026-03-14T09:21:07Z WARN request failed token={S} status=401 route=/v1/items"),
    ("markdown", True, "Here is my config:\n\n```\n{N}: {S}\n```\n\nThanks!"),
]

PEM_TYPES = ["RSA PRIVATE KEY", "EC PRIVATE KEY", "PRIVATE KEY", "ENCRYPTED PRIVATE KEY",
             "OPENSSH PRIVATE KEY", "DSA PRIVATE KEY", "PGP PRIVATE KEY BLOCK"]


def with_ctx(b, tpl, secret, subtype, env):
    tpl = tpl.replace("{N}", env).replace("{n}", env.lower())
    pre, post = tpl.split("{S}")
    b.t(pre).L(secret, "credential_secret", subtype, "critical").t(post)


def gen_secrets_pos():
    c = Corpus("secrets_positive", "sec-pos")
    r = c.r
    specs = provider_specs()
    for si, (subtype, disp, gen, env, top30, bound) in enumerate(specs):
        pool = [x for x in CTXS if x[1]] if bound else CTXS
        for v in range(4):
            name, _, tpl = pool[(si + v) % len(pool)]
            b = B()
            with_ctx(b, tpl, gen(r), subtype, env)
            c.addb(b, "block", f"{disp} format, random body. Context: {name}.", subtype_family="provider_key",
                   provider=disp, top30=top30, tags=["provider_key"] + (["context_bound"] if bound else []))
    for ptype in PEM_TYPES:
        for lines in (3, 10):
            b = B()
            b.t("Here is the key file I use for deployment, can you convert it?\n\n" if lines == 3 else "deploy_key.txt:\n")
            b.L(fake_pem(r, ptype, lines), "credential_secret", "private_key_block", "critical")
            b.t("\n\nWhat format is this?")
            c.addb(b, "block", f"PEM block '{ptype}', {lines} body lines; body decodes to a fake marker.", subtype_family="pem", tags=["pem"])
    for ptype in ["RSA PRIVATE KEY", "OPENSSH PRIVATE KEY", "PRIVATE KEY"]:
        part = "\n".join(fake_pem(r, ptype, 6).split("\n")[:4])
        b = B().t("I only pasted the top of the file:\n").L(part, "credential_secret", "private_key_block", "critical")
        c.addb(b, "block", "Truncated PEM (header + partial body, no END line); the header alone must block.", subtype_family="pem", tags=["pem", "truncated"])
    jctx = ["Authorization: Bearer {S}", "My session cookie is session={S}; Path=/; HttpOnly", "token: {S}",
            "Please decode this JWT for me: {S}", '{"id_token": "{S}"}', "export JWT={S}"]
    for i, alg in enumerate(["HS256", "RS256", "ES256", "HS384", "HS512", "RS256"]):
        pre, post = jctx[i].split("{S}")
        b = B().t(pre).L(fake_jwt(r, alg), "credential_secret", "jwt", "critical").t(post)
        c.addb(b, "block", f"JWT alg={alg}; header/payload are valid base64url JSON with invented claims, random signature.", subtype_family="jwt", tags=["jwt"])
    schemes = [("postgres", 5432), ("postgresql", 5432), ("mysql", 3306), ("mongodb", 27017), ("mongodb+srv", None),
               ("redis", 6379), ("amqp", 5672), ("mssql", 1433), ("postgres", 6543), ("mysql", 3307)]
    dctx = ["DATABASE_URL={S}", 'spring.datasource.url = "{S}"', "Why can't I connect with {S} ?", "conn = connect('{S}')", "- {S}"]
    for i, (sch, port) in enumerate(schemes):
        pw = rs(r, r.randrange(10, 20), ALNUM + "!#$%^*-_.")
        host = f"db{r.randrange(1, 9)}.{r.choice(['internal', 'prod', 'eu'])}.example.test"
        url = f"{sch}://app_{rs(r, 4, LOW)}:{pw}@{host}" + (f":{port}" if port else "") + f"/{rs(r, 6, LOW)}"
        pre, post = dctx[i % len(dctx)].split("{S}")
        b = B().t(pre).L(url, "credential_secret", "credentialed_connection_string", "critical").t(post)
        c.addb(b, "block", f"Credentialed DB URL, scheme {sch}; password random with punctuation. Label spans the whole URL.", subtype_family="db_url", tags=["db_url"])
    for i, key in enumerate(["password", "passwd", "pwd", "secret", "api_key", "auth_token", "access_token", "DB_PASSWORD", "client_secret", "api-key"]):
        val = rs(r, r.randrange(12, 22), ALNUM + "!@#%^&*-_")
        b = B()
        style = i % 4
        if style == 0:
            b.t(f"{key} = \"").L(val, "credential_secret", "password_assignment", "critical").t("\"\n")
        elif style == 1:
            b.t(f"config:\n  {key}: ").L(val, "credential_secret", "password_assignment", "critical").t("\n  timeout: 30")
        elif style == 2:
            b.t(f"my {key}: ").L(val, "credential_secret", "password_assignment", "critical").t(" - please keep it safe")
        else:
            b.t(f"export {key.upper().replace('-', '_')}=").L(val, "credential_secret", "password_assignment", "critical")
        c.addb(b, "block", "Contextual password/secret assignment; label spans the value only; value random.", subtype_family="assignment", tags=["assignment"])
    for i in range(6):
        picks = r.sample(specs, 3)
        b = B().t("Config dump:\n")
        for (subtype, disp, gen, env, top30, bound) in picks:
            b.t(f"{env}=").L(gen(r), "credential_secret", subtype, "critical").t("\n")
        c.addb(b, "block", "Three different provider secrets in one message; all must be found.", subtype_family="multi", tags=["multi"])
    return c


def gen_secrets_neg():
    c = Corpus("secrets_negative", "sec-neg")
    r = c.r

    def N(t, notes, tags):
        c.add(t, [], "allow", notes, tags=tags + ["hard_negative"])

    for _ in range(6):
        N(f"The checksum for the release is {rs(r, 64, HEXL)} (sha256).", "SHA-256 digest, not a credential.", ["hash"])
    for _ in range(3):
        N(f"git checkout {rs(r, 40, HEXL)}", "40-hex git commit id.", ["hash"])
        N(f"md5: {rs(r, 32, HEXL)}  file.tar.gz", "MD5 digest.", ["hash"])
    for _ in range(5):
        u = f"{rs(r, 8, HEXL)}-{rs(r, 4, HEXL)}-4{rs(r, 3, HEXL)}-{r.choice('89ab')}{rs(r, 3, HEXL)}-{rs(r, 12, HEXL)}"
        N(f"request_id={u} order_id={u}", "UUID v4 identifiers.", ["uuid"])
    for _ in range(4):
        raw = r.randbytes(r.randrange(60, 140))
        N(f"integrity: sha512-{base64.b64encode(hashlib.sha512(raw).digest()).decode()}", "Lockfile SRI hash; base64 of a digest.", ["base64", "hash"])
    for t in ["Please summarise the quarterly report in three bullet points.", "How do I rotate an API key safely?",
              "The key is to keep each function small and well named.", "OpenAI keys start with sk- but never paste yours here.",
              "Our ssh-agent forwarding setup uses a hardware token.", "GitHub tokens begin with ghp_ followed by characters.",
              "Add the risk-assessment-framework-document-template to the wiki page for the task-management-system-overview."]:
        N(t, "Ordinary prose mentioning key vocabulary or kebab-case words; no secret value.", ["prose"])
    for i in range(4):
        txt = "Hello from the Aegis test-suite, this is ordinary text number %d with some padding words. " % i
        N("Decode: " + base64.b64encode((txt * 2).encode()).decode(), "Base64 of ordinary English text.", ["base64"])
    for t in ["OPENAI_API_KEY=sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx", "api_key = \"YOUR_API_KEY_HERE\"", "token: <your-token>",
              "ghp_" + "x" * 36, "export AWS_SECRET_ACCESS_KEY=REPLACE_ME", "password = os.environ[\"DB_PASSWORD\"]",
              "password: <redacted>", "secret = getSecret()", "postgres://user:${DB_PASSWORD}@db.example.test/app",
              "postgres://db.example.test:5432/app", "postgres://readonly@db.example.test/app", "api_key = \"changeme\"",
              "sk-" + "0" * 40]:
        N(t, "Template/placeholder/variable reference or no embedded credential.", ["placeholder"])
    pubbody = base64.b64encode(b"AEGIS-SYNTHETIC-PUBLIC" + r.randbytes(80)).decode()
    N("-----BEGIN PUBLIC KEY-----\n" + pubbody[:64] + "\n" + pubbody[64:] + "\n-----END PUBLIC KEY-----", "PEM public key; not secret.", ["pem", "public"])
    N("-----BEGIN CERTIFICATE-----\n" + base64.b64encode(b"AEGIS-SYNTHETIC-CERT" + r.randbytes(40)).decode() + "\n-----END CERTIFICATE-----", "PEM certificate; public.", ["pem", "public"])
    N("ssh-ed25519 AAAAC3NzaC1lZDI1NTE5" + rs(r, 36, B64STD) + " user@host.example.test", "SSH public key.", ["public"])
    N("Stripe publishable key pk_live_" + rs(r, 24) + " is safe to embed in the page.", "Publishable key is public by design.", ["public"])
    # Coding-profile exemptions (OPEN-06): exact well-known documented dummy values only.
    exact = [("AWS documented example access key id", "AKIAIOSFODNN7EXAMPLE", "aws_access_key"),
             ("AWS documented example access key id (alt)", "AKIAI44QH8DHBEXAMPLE", "aws_access_key"),
             ("AWS documented example secret key", "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY", "aws_secret_key")]
    for desc, val, sub in exact:
        c.add("In the docs they use " + val, [], "allow",
              f"{desc}. OPEN-06/FR-DET-6: exact well-known pattern, exempt ONLY in the Coding profile; no entropy or user-regex exemptions.",
              profile="coding", tags=["dummy_key", "coding_exemption"], exemption="OPEN-06")
        b = B().t("In the docs they use ").L(val, "credential_secret", sub, "critical")
        c.add(b.s, b.labels, "block", "Twin of the previous Coding-profile record: same text under any other profile is a secret and blocks.",
              profile="default", tags=["dummy_key", "coding_exemption_twin"], exemption="OPEN-06")
    for _ in range(3):
        b = B().t("This is just a test key: ").L("AKIA" + rs(r, 16, UPDIG), "credential_secret", "aws_access_key", "critical")
        c.add(b.s, b.labels, "block", "Described as 'test' but not an exact well-known dummy: Coding profile must still block (no free-text/entropy exemption).",
              profile="coding", tags=["dummy_key_not_exact", "must_block_in_coding"])
    for _ in range(3):
        b = B().t("// fake key for unit tests: ").L("sk_live_" + rs(r, 24), "credential_secret", "stripe_live_key", "critical")
        c.add(b.s, b.labels, "block", "Comment claims it is fake; a live-format key is still blocked (no comment-based exemption).",
              profile="coding", tags=["dummy_key_not_exact", "must_block_in_coding"])
    return c
