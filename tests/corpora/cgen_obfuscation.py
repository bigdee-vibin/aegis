"""Obfuscation corpus (S0.4 part c): encoded / hidden / split sensitive values."""
import base64
import urllib.parse

from cgen_common import B, Corpus, DIG, address, luhn_ok, person, email_for, provider_specs, rs, UPDIG

ZW = ["​", "‌", "‍", "⁠", "﻿", "­"]
CYR = {"a": "а", "e": "е", "o": "о", "p": "р", "c": "с", "x": "х"}


def enc_base64(s, r):
    return base64.b64encode(s.encode()).decode()


def enc_b64url(s, r):
    return base64.urlsafe_b64encode(s.encode()).decode().rstrip("=")


def enc_hex(s, r):
    return s.encode().hex()


def enc_double_b64(s, r):
    return base64.b64encode(base64.b64encode(s.encode())).decode()


def enc_pct_full(s, r):
    return "".join(f"%{b:02X}" for b in s.encode())


def enc_pct_partial(s, r):
    return urllib.parse.quote(s, safe="".join(c for c in s if c.isalnum()))


def enc_zw_one(s, r):
    return "​".join(s)


def enc_zw_mixed(s, r):
    out = []
    for i, ch in enumerate(s):
        out.append(ch)
        if i % 3 == 2:
            out.append(r.choice(ZW))
    return "".join(out)


def enc_fullwidth(s, r):
    return "".join(chr(ord(c) + 0xFEE0) if 0x21 <= ord(c) <= 0x7E else c for c in s)


def enc_html(s, r):
    return "".join(f"&#x{ord(c):x};" for c in s)


def enc_spaced(s, r):
    return " ".join(s)


def enc_wrapped(s, r):
    return "\n".join(s[i:i + 16] for i in range(0, len(s), 16))


def enc_homoglyph(s, r):
    return "".join(CYR.get(c, c) if (i > 6 and c in CYR and r.random() < 0.6) else c for i, c in enumerate(s))


ENCODINGS = [
    ("base64", enc_base64, "Decode this and use it: {S}", "easy"),
    ("base64url_nopad", enc_b64url, "token (b64url): {S}", "easy"),
    ("hex", enc_hex, "hex dump of the key: {S}", "medium"),
    ("double_base64", enc_double_b64, "payload={S}", "hard"),
    ("percent_full", enc_pct_full, "https://api.example.test/v1/items?api_key={S}&limit=5", "medium"),
    ("percent_partial", enc_pct_partial, "GET /v1/items?key={S} HTTP/1.1", "easy"),
    ("zero_width_u200b", enc_zw_one, "My key: {S}", "medium"),
    ("zero_width_mixed", enc_zw_mixed, "Use this credential: {S}", "hard"),
    ("fullwidth", enc_fullwidth, "Key： {S}", "medium"),
    ("html_entities", enc_html, "<input value=\"{S}\">", "medium"),
    ("spaced_chars", enc_spaced, "k e y : {S}", "hard"),
    ("line_wrapped", enc_wrapped, "Key wrapped by my editor:\n{S}\nend", "medium"),
]


def gen_obfuscation():
    c = Corpus("obfuscation", "obf")
    r = c.r
    specs = [s for s in provider_specs() if s[5] is False][:12]
    for i, (subtype, disp, gen, env, top30, _bound) in enumerate(specs):
        for j in range(8):
            ename, fn, tpl, diff = ENCODINGS[(i + j) % len(ENCODINGS)]
            secret = gen(r)
            surface = fn(secret, r)
            pre, post = tpl.split("{S}")
            b = B().t(pre).L(surface, "credential_secret", subtype, "critical", obfuscation=ename, canonical=secret).t(post)
            c.addb(b, "block", f"{disp} key obfuscated by {ename}; canonical value in label.canonical. Detection requires decoding/normalisation.",
                   difficulty=diff, tags=["obfuscated", ename, "secret"])
    # homoglyph stretch cases
    for i in range(4):
        subtype, disp, gen, env, *_ = specs[i * 2]
        secret = gen(r)
        surface = enc_homoglyph(secret, r)
        b = B().t("fyi ").L(surface, "credential_secret", subtype, "critical", obfuscation="cyrillic_homoglyph", canonical=secret)
        c.addb(b, "block", "Latin letters partly swapped for Cyrillic lookalikes. STRETCH goal: NFKC does not fold these; confusable-skeleton mapping needed.",
               difficulty="stretch", tags=["obfuscated", "cyrillic_homoglyph", "secret", "stretch"])
    # Basic-auth credential in base64
    for _ in range(3):
        cred = f"svc_{rs(r, 4, 'abcdefghij')}:{rs(r, 18)}"
        blob = base64.b64encode(cred.encode()).decode()
        b = B().t("Authorization: Basic ").L(blob, "credential_secret", "basic_auth_credential", "critical", obfuscation="base64", canonical=cred)
        c.addb(b, "block", "HTTP Basic credential (user:password) base64 encoded.", difficulty="easy", tags=["obfuscated", "base64", "secret"])
    # splits across messages / fields / code
    for i, (subtype, disp, gen, env, *_x) in enumerate(specs[:8]):
        secret = gen(r)
        k = len(secret) // 2
        h1, h2 = secret[:k], secret[k:]
        b = B().t("My ").t(disp).t(" key starts with ").L(h1, "credential_secret", subtype, "critical", split_group="g1", fragment=1)
        b.t("\n").t("and the rest is ").L(h2, "credential_secret", subtype, "critical", split_group="g1", fragment=2)
        c.add(b.s, b.labels, "block", "Secret split across two chat messages (text = parts joined by newline). Detector must reassemble using conversation context.",
              parts=[{"role": "user", "text": p} for p in b.s.split("\n")], canonical=secret, difficulty="hard", tags=["split", "messages", "secret"])
    for i, (subtype, disp, gen, env, *_x) in enumerate(specs[8:12]):
        secret = gen(r)
        a, m, z = secret[:8], secret[8:len(secret) // 2 + 4], secret[len(secret) // 2 + 4:]
        b = B().t("part 1: ").L(a, "credential_secret", subtype, "critical", split_group="g1", fragment=1).t("\n")
        b.t("part 2: ").L(m, "credential_secret", subtype, "critical", split_group="g1", fragment=2).t("\n")
        b.t("part 3: ").L(z, "credential_secret", subtype, "critical", split_group="g1", fragment=3)
        c.add(b.s, b.labels, "block", "Secret split across three messages.", parts=[{"role": "user", "text": p} for p in b.s.split("\n")],
              canonical=secret, difficulty="hard", tags=["split", "messages", "secret"])
    for i, (subtype, disp, gen, env, *_x) in enumerate(specs[:4]):
        secret = gen(r)
        k = r.randrange(6, 12)
        b = B().t('{"prefix": "').L(secret[:k], "credential_secret", subtype, "critical", split_group="g1", fragment=1)
        b.t('", "suffix": "').L(secret[k:], "credential_secret", subtype, "critical", split_group="g1", fragment=2).t('"}')
        c.addb(b, "block", "Secret split across two JSON fields; concatenation yields the key.", canonical=secret, difficulty="hard", tags=["split", "fields", "secret"])
    for i, (subtype, disp, gen, env, *_x) in enumerate(specs[4:8]):
        secret = gen(r)
        k, k2 = len(secret) // 3, 2 * len(secret) // 3
        b = B().t("key = \"").L(secret[:k], "credential_secret", subtype, "critical", split_group="g1", fragment=1).t('" + "')
        b.L(secret[k:k2], "credential_secret", subtype, "critical", split_group="g1", fragment=2).t('" + "').L(secret[k2:], "credential_secret", subtype, "critical", split_group="g1", fragment=3).t('"')
        c.addb(b, "block", "Secret split via string concatenation in code.", canonical=secret, difficulty="hard", tags=["split", "code_concat", "secret"])
    # PII obfuscation
    pii_enc = [("zero_width_mixed", enc_zw_mixed), ("fullwidth", enc_fullwidth), ("percent_full", enc_pct_full), ("base64", enc_base64), ("html_entities", enc_html)]
    for i in range(10):
        first, last = person(r)
        em = email_for(r, first, last)
        ename, fn = pii_enc[i % len(pii_enc)]
        b = B().t("Mail ").L(fn(em, r), "identity", "email", "low", obfuscation=ename, canonical=em).t(" please.")
        c.addb(b, None, f"Email obfuscated by {ename}.", difficulty="medium", tags=["obfuscated", ename, "pii", "email"])
    for em_style in range(4):
        first, last = person(r)
        loc = f"{first.lower()}.{last.lower()}"
        dom = r.choice(["example.com", "example.org"])
        surface = [f"{loc} (at) {dom.replace('.', ' (dot) ')}", f"{loc} [at] {dom.replace('.', ' [dot] ')}", f"{loc}%40{dom}", f"{loc}＠{dom}"][em_style]
        b = B().t("write to ").L(surface, "identity", "email", "low", obfuscation=["at_dot_words", "at_dot_brackets", "percent_at", "fullwidth_at"][em_style], canonical=f"{loc}@{dom}")
        c.addb(b, None, "Common human-written email obfuscation.", difficulty="medium", tags=["obfuscated", "pii", "email"])
    for i in range(4):
        d = f"+1 212 555 01{r.randrange(0, 100):02d}"
        surface = [enc_fullwidth(d, r), enc_zw_mixed(d, r), " ".join(d.replace(" ", "")), enc_pct_full(d, r)][i]
        b = B().t("call ").L(surface, "identity", "phone", "medium", obfuscation=["fullwidth", "zero_width_mixed", "spaced_chars", "percent_full"][i], canonical=d)
        c.addb(b, None, "Phone number obfuscated (reserved 555-01xx range).", difficulty="medium", tags=["obfuscated", "pii", "phone"])
    for i, pan in enumerate(["4111111111111111", "5555555555554444", "378282246310005", "6011111111111117"]):
        assert luhn_ok(pan)
        surface = [enc_fullwidth(pan, r), enc_zw_mixed(" ".join([pan[:4], pan[4:8], pan[8:12], pan[12:]]), r), enc_base64(pan, r), enc_hex(pan, r)][i]
        b = B().t("card: ").L(surface, "financial", "card_number", "high", obfuscation=["fullwidth", "zero_width_mixed", "base64", "hex"][i], canonical=pan)
        c.addb(b, None, "Documented test PAN, obfuscated.", difficulty="medium", tags=["obfuscated", "pii", "card"])
    # card split across messages
    for pan in ["4242424242424242", "4012888888881881"]:
        b = B().t("first eight digits of my card: ").L(pan[:8], "financial", "card_number", "high", split_group="g1", fragment=1).t("\n")
        b.t("last eight: ").L(pan[8:], "financial", "card_number", "high", split_group="g1", fragment=2)
        c.add(b.s, b.labels, "review", "Documented test PAN split across two messages.", parts=[{"role": "user", "text": p} for p in b.s.split("\n")],
              canonical=pan, difficulty="hard", tags=["split", "messages", "pii", "card"])
    # obfuscation negatives
    for _ in range(4):
        txt = "ordinary words " * 6
        c.add("Decode: " + base64.b64encode(txt.encode()).decode(), [], "allow", "Base64 of ordinary words: decoding must not create a finding.", tags=["obfuscated", "hard_negative", "base64"])
    c.add("Smart​phone, co​operate and re​activate use a zero-width space only for line-breaking hints.", [], "allow", "Benign zero-width characters in prose (no secret): normalisation must not create findings.", tags=["obfuscated", "hard_negative", "zero_width"])
    c.add("Ｈｅｌｌｏ ｗｏｒｌｄ – full-width Latin text with no sensitive data.", [], "allow", "Full-width prose, no sensitive data.", tags=["obfuscated", "hard_negative", "fullwidth"])
    c.add("Path is C:%5CUsers%5Cdemo%5Cnotes.txt and the query is q=hello%20world", [], "allow", "Percent-encoded path and query with no sensitive data.", tags=["obfuscated", "hard_negative", "percent"])
    c.add("&#x48;&#x65;&#x6c;&#x6c;&#x6f; entity-encoded greeting.", [], "allow", "HTML-entity-encoded greeting, no sensitive data.", tags=["obfuscated", "hard_negative", "html_entities"])
    return c
