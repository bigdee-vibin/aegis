"""Preservation-check corpus (S0.4 part f): numbers, dates, negations must survive sanitisation.

Each record has PII labels (to be replaced) plus `preservation.must_preserve`
tokens that an acceptable rewrite MUST keep verbatim, an `acceptable_rewrite`,
and `violating_rewrites` that each break at least one must-preserve token.
"""
from cgen_common import B, Corpus, email_for, person, rs

INVERT = {"do not": "do", "never": "always", "must not": "must", "not": "", "without": "with", "cannot": "can",
          "except": "including", "unless": "if", "no more than": "more than", "at least": "at most", "only": "also"}


def mutate(rewrite, item):
    text, kind = item["text"], item["kind"]
    if kind == "number":
        for i, ch in enumerate(text):
            if ch.isdigit():
                new = text[:i] + str((int(ch) + 1) % 10) + text[i + 1:]
                return rewrite.replace(text, new, 1)
    if kind in ("negation", "exception", "comparator"):
        return rewrite.replace(text, INVERT.get(text.lower(), ""), 1)
    return rewrite.replace(text, "", 1)


MONEY = ["$1,204.50", "EUR 3,450", "£250,000", "$87.10", "INR 56,000", "CAD 9,999.99"]
DATES = ["2026-03-15", "15 March 2026", "March 3rd", "2027-01-31", "31/12/2026", "Tuesday 9 June"]


def T1(r):
    f, l = person(r)
    b = B().t("Draft a reminder to ").L(f"{f} {l}", "identity", "person_name", "low", "[PERSON_1]")
    b.t(": the invoice for ").K(r.choice(MONEY), "number").t(" is due on ").K(r.choice(DATES), "date").t(", and we ")
    b.K("do not", "negation").t(" apply the late fee ").K("unless", "exception").t(" payment is ").K(f"{r.randrange(15, 60)} days", "number")
    b.t(" overdue. Reply to ").L(email_for(r, f, l), "identity", "email", "low", "[EMAIL_1]").t(".")
    return b


def T2(r):
    f, l = person(r)
    b = B().t("Ask ").L(f"{f} {l}", "identity", "person_name", "low", "[PERSON_1]").t(" to restart the service on port ").K(str(r.choice([8443, 9090, 5001])), "number")
    b.t(" only, ").K("never", "negation").t(" on port ").K("443", "number").t(". Version ").K(f"v{r.randrange(1, 9)}.{r.randrange(0, 9)}.{r.randrange(0, 20)}", "number")
    b.t(" is the last build that is safe; ").K("do not", "negation").t(" deploy v3.x.")
    return b


def T3(r):
    f, l = person(r)
    d1, d2 = r.sample(DATES, 2)
    b = B().t("The audit window for ").L(f"{f} {l}", "identity", "person_name", "low", "[PERSON_1]").t(" runs from ").K(d1, "date").t(" to ").K(d2, "date")
    b.t(", ").K("except", "exception").t(" public holidays; budget is ").K(r.choice(MONEY), "number").t(" ").K("at most", "comparator").t(".")
    return b


def T4(r):
    f, l = person(r)
    b = B().t("Please book ").L(f"{f} {l}", "identity", "person_name", "low", "[PERSON_1]").t(" for ").K(f"{r.randrange(9, 17):02d}:{r.choice(['00', '30', '45'])} UTC", "date")
    b.t(", ").K(f"{r.choice([30, 45, 90])} minutes", "number").t(" maximum, and ").K("not before", "negation").t(" ").K("09:00", "date").t(". Call ")
    b.L(f"+1 212-555-01{r.randrange(0, 100):02d}", "identity", "phone", "medium", "[PHONE_1]").t(" if anything changes.")
    return b


def T5(r):
    f, l = person(r)
    b = B().t("Patient notes for ").L(f"{f} {l}", "identity", "person_name", "low", "[PERSON_1]").t(": the sample weighed ").K(f"{r.randrange(10, 99)}.{r.randrange(1, 9)} mg", "number")
    b.t("; the result was ").K("not", "negation").t(" significant (p = ").K(f"0.0{r.randrange(1, 9)}", "number").t("); repeat on ").K(r.choice(DATES), "date").t(".")
    return b


def T6(r):
    f, l = person(r)
    b = B().t("Contract with ").L(f"{f} {l}", "identity", "person_name", "low", "[PERSON_1]").t(" excludes liability above ").K(r.choice(MONEY), "number")
    b.t(" and ").K("cannot", "negation").t(" be terminated ").K("without", "negation").t(" ").K(f"{r.choice([30, 60, 90])} days", "number").t(" notice. Email ")
    b.L(email_for(r, f, l), "identity", "email", "low", "[EMAIL_1]").t(" for the signed copy.")
    return b


def T7(r):
    f, l = person(r)
    b = B().t("Refund ").L(f"{f} {l}", "identity", "person_name", "low", "[PERSON_1]").t(" the ").K(r.choice(MONEY), "number").t(" paid on ").K(r.choice(DATES), "date")
    b.t(" to IBAN ").L("DE89 3704 0044 0532 0130 00", "financial", "iban", "high", "[IBAN_1]").t(" ").K("no more than", "comparator").t(" ").K(f"{r.randrange(2, 9)} times", "number").t(".")
    return b


def T8(r):
    f, l = person(r)
    b = B().t("The deadline for ").L(f"{f} {l}", "identity", "person_name", "low", "[PERSON_1]").t(" is ").K(r.choice(DATES), "date").t("; the quota is ").K("at least", "comparator")
    b.t(" ").K(f"{r.randrange(5, 50)}%", "number").t(" and ").K("never", "negation").t(" ").K(f"{r.randrange(2, 9)}", "number").t(" retries per job.")
    return b


TEMPLATES = [T1, T2, T3, T4, T5, T6, T7, T8]


def gen_preservation():
    c = Corpus("preservation", "pres")
    r = c.r
    for i in range(64):
        b = TEMPLATES[i % len(TEMPLATES)](r)
        items = b.pres
        pick = [0, len(items) - 1] if len(items) > 1 else [0]
        viol = []
        for idx in pick:
            v = mutate(b.rw, items[idx])
            assert v != b.rw
            viol.append({"text": v, "violates": idx})
        row = c.addb(b, "review", "Invented people/values. A transform must replace labelled PII and keep every must_preserve token verbatim (FR-CTX-1, S2.4).",
                     tags=["preservation"] + sorted({it['kind'] for it in items}),
                     preservation={"must_preserve": items, "acceptable_rewrite": b.rw, "violating_rewrites": viol})
        # self-check
        for it in items:
            assert it["text"] in b.rw, (row["id"], it)
        for v in viol:
            assert items[v["violates"]]["text"] not in v["text"] or v["text"].count(items[v["violates"]]["text"]) < b.rw.count(items[v["violates"]]["text"]), row["id"]
    return c
