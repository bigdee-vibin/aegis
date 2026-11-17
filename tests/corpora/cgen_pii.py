"""PII corpora (S0.4 part b): identity + financial positives, negatives, other categories."""
from cgen_common import (B, Corpus, DIG, HEXL, LOW, UP, UPDIG, address, bsn_ok, cpf_digits, dni_letter, email_for,
                         iban_check, iban_ok, luhn_check_digit, luhn_ok, nric_letter, person, rs, verhoeff_digit)

CARDS = [  # publicly documented test PANs (card-brand / payment-provider documentation)
    ("Visa", "4111111111111111"), ("Visa", "4242424242424242"), ("Visa", "4012888888881881"),
    ("Mastercard", "5555555555554444"), ("Mastercard", "5105105105105100"), ("Mastercard", "2223003122003222"),
    ("Amex", "378282246310005"), ("Amex", "371449635398431"), ("Discover", "6011111111111117"),
    ("Discover", "6011000990139424"), ("Diners", "30569309025904"), ("JCB", "3530111333300000"),
]

IBAN_SPECS = [
    ("DE", lambda r: rs(r, 18, DIG)), ("GB", lambda r: rs(r, 4, UP) + rs(r, 14, DIG)),
    ("FR", lambda r: rs(r, 10, DIG) + rs(r, 11, UPDIG) + rs(r, 2, DIG)), ("ES", lambda r: rs(r, 20, DIG)),
    ("NL", lambda r: rs(r, 4, UP) + rs(r, 10, DIG)), ("IT", lambda r: rs(r, 1, UP) + rs(r, 10, DIG) + rs(r, 12, UPDIG)),
    ("BE", lambda r: rs(r, 12, DIG)), ("CH", lambda r: rs(r, 5, DIG) + rs(r, 12, UPDIG)),
    ("IE", lambda r: rs(r, 4, UP) + rs(r, 14, DIG)),
]


def fmt_pan(pan, style):
    if style == 0:
        return pan
    if len(pan) in (14, 15):
        parts = [pan[:4], pan[4:10], pan[10:]]
    else:
        parts = [pan[i:i + 4] for i in range(0, 16, 4)]
    return (" " if style == 1 else "-").join(parts)


def phones(r):
    """Variants as (region, surface, tags). Reserved/fictional ranges where a public one exists."""
    out = []
    n = r.randrange(0, 100)
    out += [("US", f"+1 212-555-01{n:02d}", ["reserved_fictional"]), ("US", f"(212) 555-01{n:02d}", ["reserved_fictional"]),
            ("US", f"212.555.01{n:02d}", ["reserved_fictional"]), ("CA", f"416-555-01{n:02d}", ["reserved_fictional"])]
    m = r.randrange(0, 1000)
    out += [("GB", f"+44 7700 900{m:03d}", ["reserved_fictional"]), ("GB", f"07700 900{m:03d}", ["reserved_fictional"]),
            ("GB", f"+44 (0)20 7946 0{r.randrange(0, 1000):03d}", ["reserved_fictional"])]
    au = r.choice(["006", "156", "157", "158", "159"])
    out += [("AU", f"0491 570 {au}", ["reserved_fictional"]), ("AU", f"+61 491 570 {au}", ["reserved_fictional"])]
    out += [("FR", f"06 39 98 {r.randrange(0, 100):02d} {r.randrange(0, 100):02d}", ["reserved_fictional"]),
            ("FR", f"+33 6 39 98 {r.randrange(0, 100):02d} {r.randrange(0, 100):02d}", ["reserved_fictional"])]
    rnd = ["random_number"]
    out += [("DE", f"+49 151 {r.randrange(1000000, 9999999)}", rnd), ("DE", f"0151 {r.randrange(10000000, 99999999)}", rnd),
            ("IN", f"+91 {r.choice('6789')}{rs(r, 4, DIG)} {rs(r, 5, DIG)}", rnd), ("IN", f"{r.choice('6789')}{rs(r, 9, DIG)}", rnd),
            ("JP", f"090-{rs(r, 4, DIG)}-{rs(r, 4, DIG)}", rnd), ("JP", f"+81 90 {rs(r, 4, DIG)} {rs(r, 4, DIG)}", rnd),
            ("BR", f"+55 11 9{rs(r, 4, DIG)}-{rs(r, 4, DIG)}", rnd), ("SG", f"+65 {r.choice('89')}{rs(r, 3, DIG)} {rs(r, 4, DIG)}", rnd),
            ("ZA", f"+27 82 {rs(r, 3, DIG)} {rs(r, 4, DIG)}", rnd), ("ES", f"+34 6{rs(r, 2, DIG)} {rs(r, 3, DIG)} {rs(r, 3, DIG)}", rnd),
            ("NL", f"+31 6 {rs(r, 8, DIG)}", rnd), ("SE", f"+46 70 {rs(r, 3, DIG)} {rs(r, 2, DIG)} {rs(r, 2, DIG)}", rnd)]
    return out


def national_ids(r):
    """(country, subtype, value, tags). Checksums valid by construction."""
    out = []
    for _ in range(3):
        out.append(("US", "us_ssn", f"{r.randrange(900, 1000)}-{r.randrange(1, 100):02d}-{r.randrange(1, 10000):04d}", ["unissued_range"]))
    for _ in range(3):
        out.append(("GB", "uk_nino", f"QQ {r.randrange(10, 100)} {r.randrange(10, 100)} {r.randrange(10, 100)} {r.choice('ABCD')}", ["unissued_range"]))
    for _ in range(3):
        base = "0" + rs(r, 7, DIG)
        sin = base + luhn_check_digit(base)
        out.append(("CA", "ca_sin", f"{sin[:3]} {sin[3:6]} {sin[6:]}", ["unissued_range", "luhn"]))
    for _ in range(3):
        base = str(r.randrange(2, 10)) + rs(r, 10, DIG)
        a = base + verhoeff_digit(base)
        out.append(("IN", "in_aadhaar", f"{a[:4]} {a[4:8]} {a[8:]}", ["verhoeff", "random_number"]))
    for _ in range(3):
        n = r.randrange(10 ** 7, 10 ** 8)
        out.append(("ES", "es_dni", f"{n}{dni_letter(n)}", ["mod23", "random_number"]))
    for _ in range(3):
        while True:
            n8 = rs(r, 8, DIG)
            ds = [d for d in range(10) if bsn_ok(n8 + str(d))]
            if ds:
                break
        out.append(("NL", "nl_bsn", n8 + str(ds[0]), ["mod11", "random_number"]))
    for _ in range(3):
        s = cpf_digits(rs(r, 9, DIG))
        out.append(("BR", "br_cpf", f"{s[:3]}.{s[3:6]}.{s[6:9]}-{s[9:]}", ["cpf_check", "random_number"]))
    for _ in range(3):
        d7 = rs(r, 7, DIG)
        out.append(("SG", "sg_nric", "S" + d7 + nric_letter(d7), ["nric_check", "random_number"]))
    for _ in range(3):
        base = r.choice("12") + f"{r.randrange(40, 99):02d}{r.randrange(1, 13):02d}{r.randrange(1, 96):02d}{rs(r, 6, DIG)}"
        key = 97 - int(base) % 97
        out.append(("FR", "fr_insee", f"{base[0]} {base[1:3]} {base[3:5]} {base[5:7]} {base[7:10]} {base[10:13]} {key:02d}", ["mod97", "random_number"]))
    for _ in range(3):
        nine = f"{r.randrange(50, 99):02d}{r.randrange(1, 13):02d}{r.randrange(1, 29):02d}{rs(r, 3, DIG)}"
        full = nine + luhn_check_digit(nine)
        out.append(("SE", "se_personnummer", full[:6] + "-" + full[6:], ["luhn", "random_number"]))
    return out


def make_iban(r, cc, gen):
    bban = gen(r)
    iban = cc + iban_check(cc, bban) + bban
    assert iban_ok(iban)
    return iban


def fmt_iban(iban, grouped):
    return " ".join(iban[i:i + 4] for i in range(0, len(iban), 4)) if grouped else iban


def gen_pii_pos():
    c = Corpus("pii_positive", "pii-pos")
    r = c.r
    ectx = ["Please send the report to {S} by Friday.", "Contact: {S}", "From: {S}\nSubject: invoice", '{"email": "{S}"}',
            "Can you write a polite reply to {S} declining the offer?", "user,{S},active"]
    for i in range(30):
        first, last = person(r)
        em = email_for(r, first, last)
        if i == 27:
            em = f"café.{first.lower()}@example.org"
        pre, post = ectx[i % len(ectx)].split("{S}")
        b = B().t(pre).L(em, "identity", "email", "low").t(post)
        c.addb(b, None, "Invented name; reserved domain (example.*). Detectors must still flag: reserved-domain handling is policy, not tested here.",
               pii_type="email", tags=["email"] + (["unicode_local_part"] if i == 27 else []))
    pctx = ["Call me on {S} after 6pm.", "Phone: {S}", "My number is {S}, text only please.", "tel:{S}", "Reach {S} for the delivery."]
    k = 0
    for _rep in range(2):
        for region, surface, tags in phones(r):
            pre, post = pctx[k % len(pctx)].split("{S}")
            k += 1
            b = B().t(pre).L(surface, "identity", "phone", "medium", region=region).t(post)
            c.addb(b, None, f"Phone, region {region}. " + ("Reserved/fictional range." if "reserved_fictional" in tags else "Random digits in a plausible national format (not taken from any source)."),
                   pii_type="phone", region=region, tags=["phone"] + tags)
    nctx = ["My ID number is {S}.", "ID: {S}", "Please verify {S} against the form.", "tax id {S}"]
    for k, (cc, sub, val, tags) in enumerate(national_ids(r)):
        pre, post = nctx[k % len(nctx)].split("{S}")
        b = B().t(pre).L(val, "identity", sub, "high", region=cc).t(post)
        note = "Checksum valid by construction (see LABELLING_SPEC). "
        if "unissued_range" in tags:
            note += "Value lies in a range never issued to people; detectors that reject unissued ranges should skip records tagged unissued_range."
        c.addb(b, None, note, pii_type="national_id", region=cc, tags=["national_id"] + tags)
    cctx = ["Charge my card {S} exp 09/29 cvv 123.", "Card number: {S}", "I paid with {S} yesterday, was I double charged?", "pan={S}"]
    k = 0
    for style in range(3):
        for brand, pan in CARDS:
            assert luhn_ok(pan)
            pre, post = cctx[k % len(cctx)].split("{S}")
            k += 1
            b = B().t(pre).L(fmt_pan(pan, style), "financial", "card_number", "high", brand=brand).t(post)
            c.addb(b, None, f"{brand} publicly documented test PAN, Luhn-valid; format style {style} (0 plain, 1 spaces, 2 dashes).",
                   pii_type="card", tags=["card", "luhn", "documented_test_pan"])
    ictx = ["Please wire to IBAN {S}.", "IBAN: {S}", "Refund to {S} within 5 days."]
    k = 0
    for cc, gen in IBAN_SPECS:
        for grouped in (False, True):
            iban = make_iban(r, cc, gen)
            pre, post = ictx[k % len(ictx)].split("{S}")
            k += 1
            b = B().t(pre).L(fmt_iban(iban, grouped), "financial", "iban", "high", region=cc).t(post)
            c.addb(b, None, f"{cc} IBAN, mod-97 valid, random BBAN (bank/branch codes not taken from any real source).", pii_type="iban", region=cc,
                   tags=["iban", "mod97"] + (["grouped"] if grouped else []))
    for _ in range(14):
        first, last = person(r)
        b = B().t(r.choice(["Write a reference letter for ", "Hi, my name is ", "Please introduce ", "Customer: "]))
        b.L(f"{first} {last}", "identity", "person_name", "low").t(r.choice([", who has worked with us for six years.", ".", " to the team.", ""]))
        c.addb(b, None, "Invented name (syllable-generated); may coincide with a real person only by chance.", pii_type="name", tags=["name"])
    for _ in range(14):
        b = B().t(r.choice(["Ship it to ", "My address is ", "Deliver to: ", "She lives at "])).L(address(r), "identity", "address", "medium")
        b.t(r.choice([".", " please.", "", " (side door)."]))
        c.addb(b, None, "Invented street (syllable-generated) in a fictional town; postal code random.", pii_type="address", tags=["address"])
    for _ in range(10):
        first, last = person(r)
        ph = r.choice(phones(r))
        dob = f"{r.randrange(1950, 2004)}-{r.randrange(1, 13):02d}-{r.randrange(1, 29):02d}"
        b = B().t("Customer record:\nName: ").L(f"{first} {last}", "identity", "person_name", "low")
        b.t("\nEmail: ").L(email_for(r, first, last), "identity", "email", "low").t("\nPhone: ").L(ph[1], "identity", "phone", "medium", region=ph[0])
        b.t("\nAddress: ").L(address(r), "identity", "address", "medium").t("\nDOB: ").L(dob, "identity", "date_of_birth", "medium")
        c.addb(b, None, "Combined invented record: five labelled fields; action is the max over labels.", pii_type="record", tags=["record", "multi"])
    return c


def gen_pii_neg():
    c = Corpus("pii_negative", "pii-neg")
    r = c.r

    def N(t, notes, tags):
        c.add(t, [], "allow", notes, tags=["hard_negative"] + tags)

    for _ in range(6):
        d = rs(r, 15, DIG)
        bad = next(d + str(k) for k in range(10) if not luhn_ok(d + str(k)))
        N(f"Order reference {bad[:4]} {bad[4:8]} {bad[8:12]} {bad[12:]} shipped today.", "16 digits grouped like a card but Luhn-INVALID.", ["card_lookalike", "luhn_invalid"])
    for _ in range(4):
        N(f"Timestamp {r.randrange(1700000000, 1790000000)} and build number {r.randrange(10 ** 9, 10 ** 10)}.", "10-digit timestamps/ids resemble phone numbers.", ["phone_lookalike"])
    for v in ["Upgrade to v12.4.1234.5678 before Friday.", "Server at 192.168.10.45 port 8080 is reachable.", "ISBN 978-3-16-148410-0 is the paperback.",
              "Coordinates 51.5072, -0.1276 are public landmarks.", "Invoice #2026-0042-1187 total 1,204.50.", "Part no. 4400-2210-9981 in stock.",
              "Call extension 4417 for the front desk.", "Meeting ID 812 3345 6677 starts at noon."]:
        N(v, "Numeric lookalike (version/IP/ISBN/coordinates/ids) with no personal data.", ["numeric_lookalike"])
    for sub, val in [("area 000", "000-12-3456"), ("area 666", "666-12-3456"), ("group 00", "123-00-4567"), ("serial 0000", "123-45-0000")]:
        N(f"Form field example: {val}", f"SSN-shaped value invalid by SSA structural rules ({sub}).", ["national_id_lookalike", "checksum_invalid"])
    for _ in range(4):
        base = "0" + rs(r, 7, DIG)
        sin = base + next(str(d) for d in range(10) if str(d) != luhn_check_digit(base))
        N(f"SIN {sin[:3]} {sin[3:6]} {sin[6:]} (typo?)", "SIN-shaped value with WRONG Luhn digit.", ["national_id_lookalike", "checksum_invalid"])
    for _ in range(3):
        base = str(r.randrange(2, 10)) + rs(r, 10, DIG)
        a = base + next(str(d) for d in range(10) if str(d) != verhoeff_digit(base))
        N(f"Number {a[:4]} {a[4:8]} {a[8:]}", "12 digits with WRONG Verhoeff check digit.", ["national_id_lookalike", "checksum_invalid"])
    for cc, gen in IBAN_SPECS[:4]:
        iban = make_iban(r, cc, gen)
        bad = iban[:2] + ("00" if iban[2:4] != "00" else "01") + iban[4:]
        assert not iban_ok(bad)
        N(f"Reference {fmt_iban(bad, True)} was rejected.", "IBAN-shaped with WRONG mod-97 check digits.", ["iban_lookalike", "checksum_invalid"])
    for t in ["Install with npm i @types/node@18.0.0 and lodash@4.17.21.", "git remote add origin git@github.com:acme/widgets.git",
              "Use the @mention syntax to notify teammates, e.g. @channel.", "import { Component } from '@angular/core';",
              "ssh deploy@${HOST} 'uptime'", "Template: Dear {{first_name}}, your order ships on {{date}}."]:
        N(t, "@-syntax/templating that is not an email address or person.", ["email_lookalike"])
    for t in ["The Hollow Knight soundtrack is excellent.", "Alder Street Bakery opens at 7.", "We met at Brindlemouth station last week.",
              "Marvelin is a type of fictional fabric in the novel."]:
        N(t, "Capitalised words / place names with no identifiable person (advisory; a name-dictionary hook flagging these is low-confidence).", ["name_lookalike"])
    for t in ["Our office is on the third floor.", "Main Street is closed for roadworks until Monday.", "Take the A40 towards the city centre."]:
        N(t, "Generic place reference without a specific address.", ["address_lookalike"])
    for t in ["Contact us at info at the company dot com.", "Email: [redacted]", "Phone: +1 (XXX) XXX-XXXX", "Card: **** **** **** 1111"]:
        N(t, "Already-masked or abstract contact details.", ["masked"])
    return c


def gen_other_cats():
    c = Corpus("other_categories", "oth")
    r = c.r
    for t in ["CONFIDENTIAL - INTERNAL ONLY. Do not distribute outside Fennwick Robotics Ltd (fictional).",
              "This Non-Disclosure Agreement between Quelmere Dynamics (fictional) and the recipient covers unreleased product roadmaps.",
              "Board minutes - strictly confidential: acquisition of Ostenlund Foods (fictional) at an unannounced valuation.",
              "Draft press release EMBARGOED until 2027-02-01: Vintbrook Labs (fictional) announces Series B.",
              "Salary bands FY27 (HR confidential): L4 band 71k-88k, L5 band 90k-112k (invented figures).",
              "Customer pricing sheet - NDA covered. Tier 1 discount 31%, Tier 2 discount 22% (invented).",
              "Internal memo: layoffs planned for the Brindlemouth office in Q3 (fictional), not yet communicated.",
              "Pre-release benchmark results, under embargo, for the Strommere 9 chip (fictional)."]:
        c.addb(B().L(t, "confidential", "confidential_marking", "medium"), None, "Invented company/documents; confidentiality markers and business-sensitive content.", tags=["confidential"])
    for t in ["The company confidentiality policy is reviewed annually.", "Please summarise this public press release from last year.",
              "What does 'commercial in confidence' mean in contracts?"]:
        c.add(t, [], "allow", "Discussion of confidentiality as a topic, no confidential content.", tags=["confidential", "hard_negative"])
    for _ in range(10):
        host = f"{r.choice(['build', 'db', 'vault', 'jump'])}{r.randrange(1, 40)}.{r.choice(['corp', 'internal', 'lab'])}.example.test"
        ip = f"10.{r.randrange(0, 255)}.{r.randrange(0, 255)}.{r.randrange(1, 255)}"
        b = B().t("SSH to ").L(host, "technical", "internal_hostname", "low").t(" at ").L(ip, "technical", "private_ip", "low").t(" and check the cron job.")
        c.addb(b, None, "Invented internal hostname (reserved .test TLD) and RFC1918 address.", tags=["technical"])
    for t in ["Public DNS for example.com resolves to 93.184.216.34 (documentation domain).", "localhost listens on 127.0.0.1:3000."]:
        c.add(t, [], "allow", "Public/loopback addressing; not sensitive infrastructure detail.", tags=["technical", "hard_negative"])
    for pre, post, k in [("Patient ", "was diagnosed with type 2 diabetes in 2019 and takes metformin twice daily.", "medical"),
                         ("Employee ", "received a written warning on 2026-01-12 for repeated absence; HR case 4471.", "hr"),
                         ("Applicant ", "has a spent conviction for shoplifting from 2011 and declared it on the form.", "criminal"),
                         ("Student ", "has an individual education plan for dyslexia and receives extra time in exams.", "education"),
                         ("Tenant ", "is three months behind on rent and has agreed a repayment plan.", "financial_hardship"),
                         ("Client ", "disclosed they are in therapy for anxiety and sleep disorders.", "mental_health"),
                         ("Member ", "left the union after a grievance about a disability accommodation.", "union_health"),
                         ("Patient ", "had a positive pregnancy test on 2026-02-03 and has not told her employer.", "medical")]:
        first, last = person(r)
        b = B().t(pre).L(f"{first} {last}", "identity", "person_name", "low").t(" ").L(post, "personal_record", k + "_record", "high")
        c.addb(b, None, "Invented person and invented sensitive-record statement.", tags=["personal_record"])
    for t in ["The only cardiologist at Brindlemouth General who also coaches the under-12 rowing club lives on Gorse Close.",
              "Our CFO, who joined from Ostenlund Foods last March and has three children at St Alder's, signed the memo.",
              "The sole female engineer on the night shift at Kestrel Bay depot, the one with the red bicycle.",
              "A 47-year-old man from Hallowmere with a prosthetic left leg, who works at the only bakery on Fallow Lane.",
              "The mayor's brother, a retired surveyor living above the chemist in Port Vellacott.",
              "Her flatmate, the Tuesday-night bell ringer at St Ivel's in Northwick-on-Tarn, forgot the keys."]:
        c.addb(B().L(t, "contextual_identity", "identifying_combination", "medium"), None,
               "No direct identifier but the combination of details is identifying (fictional setting).", tags=["contextual_identity"])
    for t in ["A middle-aged person from a large city who works in an office.", "Someone in the sales team mentioned the schedule.", "A teacher in my town was helpful."]:
        c.add(t, [], "allow", "Generic description; not identifying.", tags=["contextual_identity", "hard_negative"])
    return c
