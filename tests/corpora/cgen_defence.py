"""Defence-context corpus (S0.4 part e).

Everything here is invented: fictional states, units, operations and equipment.
Records are FLAGGED FOR REVIEW; they are never auto-classified as classified
(FR-DET-5). General military discussion is a negative.
"""
from cgen_common import B, Corpus, person

UNIT_ADJ = ["Ardent", "Hollow", "Granite", "Tessaly", "Ember", "Quarrel"]
UNIT_TYPE = ["Signals Battalion", "Logistics Regiment", "Engineer Company", "Transport Squadron", "Medical Detachment"]
PLACES = ["Harbour Gate", "North Ferry Landing", "Karrow Depot", "Tessaly Junction", "Brindle Pass"]
CODENAMES = ["AMBER LANTERN", "QUIET HARROW", "IRON MEADOW", "SALT WINDOW", "LONG TIDE", "PALE ORCHARD"]
STATES = ["Republic of Valmora", "Kesterland", "Union of Orrin"]
MODELS = ["VX-9", "Harrier-K", "TQ-300", "Osprey-M2", "LN-12"]


def unit(r):
    return f"{r.randrange(2, 19)}th {r.choice(UNIT_ADJ)} {r.choice(UNIT_TYPE)}"


def gen_defence():
    c = Corpus("defence", "def")
    r = c.r

    def pos(text, sub, sev="medium", action="review", **x):
        b = B().L(text, "defence", sub, sev)
        c.addb(b, action, "INVENTED content about a fictional state/unit. Flag for human review; do NOT auto-classify as classified (FR-DET-5).",
               needs_human_review=True, auto_classify=False, **x)

    for _ in range(14):
        s = r.choice(STATES)
        pos(f"Convoy {r.choice(CODENAMES)} of the {s} army departs {r.choice(PLACES)} at {r.randrange(0, 24):02d}{r.choice(['00', '30'])} local on 2026-{r.randrange(1, 13):02d}-{r.randrange(1, 29):02d}; escort strength two vehicles, route {r.choice(['Amber', 'Slate', 'Moss'])}.",
            "operational_detail", tags=["defence", "operational_detail"], context="operational_detail")
    for _ in range(14):
        pos(f"RESTRICTED - FICTIONAL EXERCISE DOCUMENT. Annex C to Operation {r.choice(CODENAMES)}: distribution limited to {unit(r)} staff; copy {r.randrange(1, 30)} of 40.",
            "restricted_document", tags=["defence", "restricted_document"], context="restricted_documents")
    for _ in range(14):
        pos(f"The {r.choice(PLACES)} substation feeds the northern hangars of the {r.choice(STATES)} air arm; the backup generator is tested every {r.choice(['Monday', 'Friday'])} at {r.randrange(5, 9):02d}:00.",
            "infrastructure", tags=["defence", "infrastructure"], context="infrastructure")
    for _ in range(14):
        f1, l1 = person(r)
        f2, l2 = person(r)
        pos(f"Roster: Capt. {f1} {l1} and Lt. {f2} {l2} posted to the {unit(r)} from 2026-{r.randrange(1, 13):02d}-01; leave frozen during exercise {r.choice(CODENAMES)}.",
            "personnel", tags=["defence", "personnel"], context="personnel")
    for _ in range(14):
        pos(f"The fictional {r.choice(MODELS)} radar of the {r.choice(STATES)} navy uses a {r.choice(['8', '12', '16'])}-channel frequency-hopping scheme and a {r.choice(['2', '4'])} second dwell (invented figures).",
            "technology", tags=["defence", "technology"], context="technology")
    # borderline: low severity, notify
    for t in ["Our unit (a fictional reserve company) has an exercise next month, so I may be slow to reply.",
              "I work in logistics for a fictional defence contractor and need help phrasing a status update.",
              "Draft a polite notice that the depot car park will be closed on Saturday (fictional garrison).",
              "Summarise this invented training schedule: Monday drill, Tuesday classroom, Wednesday range day."]:
        b = B().L(t, "defence", "defence_context", "low")
        c.addb(b, "notify", "Borderline: defence-adjacent context but no operational detail; notify only. Invented.",
               needs_human_review=False, auto_classify=False, tags=["defence", "borderline"], context="borderline")
    # negatives: general discussion
    for t in ["What were the main causes of the Peloponnesian War?",
              "Explain in general terms how NATO's collective defence clause works.",
              "Summarise the history of the Roman legion's organisation into cohorts and centuries.",
              "Help me design the rules for a naval strategy board game set in a fictional world.",
              "I am a veteran writing a CV; how do I describe leadership experience for civilian employers?",
              "Compare arguments for and against raising national defence budgets.",
              "Write a review of a war novel set in the First World War.",
              "How do military ranks map to civilian management levels in general?",
              "What is the difference between a corvette and a frigate?",
              "Explain what 'logistics is the backbone of an army' means with historical examples.",
              "Describe how radar works in general, for a school physics class.",
              "Give me a short history of the Cold War arms control treaties.",
              "Suggest a name for a fictional army in my fantasy novel.",
              "How do reservists balance civilian jobs with training obligations?",
              "What are common ethical debates around drones in warfare?",
              "Translate 'the garrison held the bridge' into French."]:
        c.add(t, [], "allow", "General military/defence discussion: not auto-classified (FR-DET-5). Subject matter alone is not severity.",
              tags=["defence", "hard_negative", "general_discussion"], context="general_discussion")
    return c
