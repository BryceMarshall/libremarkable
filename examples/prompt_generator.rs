//! Prompt Generator Module
//!
//! Generates varied handwriting prompts from dictionary words with configurable
//! punctuation and prompt types.

use rand::prelude::*;
use rand::rngs::StdRng;
use serde::Serialize;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Embedded word list - ~1500 common English words
/// Includes short, medium, and longer words with good letter coverage
pub static EMBEDDED_WORDS: &[&str] = &[
    // Common short words (2-4 chars)
    "the", "and", "for", "are", "but", "not", "you", "all", "can", "had",
    "her", "was", "one", "our", "out", "day", "get", "has", "him", "his",
    "how", "its", "may", "new", "now", "old", "see", "two", "way", "who",
    "boy", "did", "own", "say", "she", "too", "use", "big", "end", "few",
    "run", "set", "top", "try", "ask", "let", "put", "add", "ago", "air",
    "bed", "bit", "box", "car", "cut", "dog", "eat", "far", "fun", "got",
    "hat", "hot", "ice", "job", "key", "lay", "leg", "lot", "low", "map",
    "men", "mix", "oil", "pay", "red", "sea", "sit", "six", "sun", "ten",
    "war", "wet", "win", "yes", "yet", "art", "bad", "bag", "bar", "bat",
    "bus", "buy", "cap", "cat", "cup", "dry", "due", "ear", "egg", "eye",
    "fan", "fat", "fit", "fly", "gas", "gun", "guy", "hit", "hop", "hug",
    "ink", "jam", "jar", "jog", "joy", "kit", "lab", "lap", "led", "lie",
    "lip", "log", "mad", "mat", "met", "mud", "net", "nod", "nor", "not",
    "nut", "oak", "odd", "pan", "pat", "pea", "pen", "pet", "pie", "pig",
    "pin", "pit", "pop", "pot", "raw", "rid", "rip", "rob", "rod", "rot",
    "row", "rub", "rug", "sad", "sat", "sew", "shy", "sin", "sip", "sky",
    "sob", "sod", "son", "sow", "spy", "sum", "tab", "tan", "tap", "tar",
    "tax", "tea", "tie", "tin", "tip", "toe", "ton", "tub", "tug", "van",
    "wag", "wed", "wig", "wit", "won", "woo", "yam", "yap", "zap", "zip",
    // Medium words (5-7 chars)
    "about", "above", "after", "again", "being", "below", "black", "bring",
    "build", "carry", "catch", "cause", "chair", "child", "clean", "clear",
    "close", "could", "cover", "cross", "dance", "doing", "dozen", "dream",
    "drink", "drive", "earth", "eight", "enjoy", "enter", "every", "fight",
    "final", "first", "floor", "force", "found", "fresh", "front", "fruit",
    "glass", "going", "grass", "great", "green", "group", "guess", "happy",
    "heart", "heavy", "hello", "horse", "hotel", "house", "human", "image",
    "inner", "judge", "juice", "knife", "known", "large", "later", "laugh",
    "learn", "leave", "lemon", "light", "limit", "local", "lower", "lucky",
    "lunch", "magic", "major", "march", "match", "maybe", "metal", "might",
    "money", "month", "moral", "motor", "mouse", "mouth", "movie", "music",
    "never", "night", "noise", "north", "novel", "occur", "ocean", "offer",
    "often", "order", "other", "owner", "paint", "paper", "party", "peace",
    "phone", "photo", "piano", "piece", "pilot", "pitch", "place", "plain",
    "plane", "plant", "plate", "point", "power", "press", "price", "pride",
    "prime", "print", "prize", "proof", "proud", "prove", "queen", "quick",
    "quiet", "quite", "radio", "raise", "range", "rapid", "reach", "react",
    "ready", "refer", "relax", "reply", "rider", "right", "river", "robot",
    "rough", "round", "route", "royal", "scene", "score", "sense", "serve",
    "seven", "shake", "shall", "shape", "share", "sharp", "sheet", "shelf",
    "shell", "shift", "shine", "shirt", "shock", "shoot", "short", "shout",
    "sight", "silly", "since", "skill", "sleep", "slide", "small", "smart",
    "smell", "smile", "smoke", "solid", "solve", "sorry", "sound", "south",
    "space", "spare", "speak", "speed", "spend", "spite", "split", "sport",
    "staff", "stage", "stand", "start", "state", "steam", "steel", "steep",
    "stick", "still", "stock", "stone", "store", "storm", "story", "strip",
    "study", "stuff", "style", "sugar", "sunny", "super", "sweet", "swing",
    "table", "taste", "teach", "thank", "their", "theme", "there", "these",
    "thick", "thing", "think", "third", "those", "three", "throw", "tight",
    "tired", "title", "today", "tooth", "total", "touch", "tough", "tower",
    "trace", "track", "trade", "trail", "train", "trash", "treat", "trend",
    "trial", "tribe", "trick", "tried", "truck", "truly", "trust", "truth",
    "twice", "uncle", "under", "union", "unite", "until", "upper", "upset",
    "urban", "usual", "valid", "value", "video", "visit", "vital", "vocal",
    "voice", "waste", "watch", "water", "wheel", "where", "which", "while",
    "white", "whole", "whose", "woman", "world", "worry", "worse", "worst",
    "worth", "would", "write", "wrong", "wrote", "young", "youth", "zebra",
    // Longer words (8+ chars)
    "absolute", "abstract", "academic", "accident", "accurate", "achieved",
    "activity", "actually", "addition", "adequate", "advanced", "advising",
    "affected", "afternoon", "agreement", "aircraft", "allowing", "although",
    "aluminum", "american", "analysis", "announce", "answered", "anything",
    "anywhere", "apparent", "approach", "approved", "argument", "artistic",
    "assembly", "assuming", "athletic", "attached", "attacked", "attempts",
    "attended", "attorney", "audience", "automatic", "available", "bachelor",
    "backward", "bacteria", "balanced", "baseball", "bathroom", "becoming",
    "behavior", "believed", "belonged", "benefits", "birthday", "blankets",
    "blessing", "blocking", "boundary", "branches", "breaking", "breathing",
    "brighten", "bringing", "brothers", "browsing", "building", "bulletin",
    "business", "calendar", "campaign", "capacity", "captured", "cardinal",
    "careful", "carrying", "category", "cellular", "cemetery", "centered",
    "ceremony", "chairman", "chambers", "champion", "changing", "channels",
    "chapters", "charging", "charming", "checking", "chemical", "children",
    "choosing", "churches", "circular", "citizens", "claiming", "classical",
    "cleaning", "clearing", "clicking", "climbing", "clinical", "closures",
    "clothing", "clusters", "coaching", "coalition", "collapse", "colleges",
    "colonial", "combined", "comeback", "comedian", "commands", "comments",
    "commerce", "commonly", "communal", "compact", "companies", "compared",
    "compares", "competed", "complete", "composed", "compound", "computer",
    "concepts", "concerns", "conclude", "concrete", "condemns", "condition",
    "conduct", "conflict", "confused", "congress", "connects", "conquest",
    "consider", "consists", "constant", "consumer", "contacts", "contains",
    "contempt", "contents", "contexts", "continue", "contract", "contrast",
    "controls", "converts", "convince", "cooking", "copyright", "corporate",
    "correct", "corridor", "councils", "counting", "counties", "countries",
    "coverage", "covering", "creating", "creation", "creative", "creature",
    "criminal", "critical", "crossing", "cultural", "cultures", "currency",
    "customer", "database", "daughter", "deciding", "decision", "declared",
    "decrease", "defaults", "defeated", "defender", "defining", "definite",
    "delivery", "demanded", "democrat", "denoting", "departed", "deployed",
    "deposits", "depressed", "describe", "deserves", "designed", "designer",
    "desiring", "desperate", "detailed", "detector", "develops", "diabetes",
    "diagonal", "dialogue", "diameter", "diamonds", "dictated", "different",
    "difficult", "dilemmas", "dimension", "dinosaur", "directed", "director",
    "disabled", "disaster", "disclose", "discount", "discover", "discrete",
    "discusses", "diseases", "disguise", "dispatch", "displays", "disputed",
    "distinct", "districts", "disturbs", "dividend", "division", "divorced",
    "doctrine", "document", "domestic", "dominant", "donation", "download",
    "downtown", "dramatic", "drawings", "dressing", "drinking", "dropping",
    "duration", "dynamics", "earnings", "economic", "educated", "educator",
    "effective", "efficiency", "eighteen", "election", "electric", "electron",
    "elements", "elephant", "eligible", "embedded", "emerging", "emission",
    "emotions", "emphasis", "employee", "employer", "employing", "enabling",
    "enclosed", "encoding", "encounter", "enduring", "engaging", "engineer",
    "enjoying", "enormous", "enrolled", "ensuring", "entering", "entirely",
    "entitled", "entrance", "envelope", "equality", "equation", "equipped",
    "escaping", "essence", "establish", "estimate", "ethernet", "european",
    "evaluate", "evenings", "everyday", "everyone", "evidence", "evolving",
    "examines", "examples", "exceeded", "exchange", "exciting", "excluded",
    "executed", "exercise", "exhibits", "existing", "expanded", "expected",
    "expenses", "expensive", "experiment", "explains", "explicit", "explored",
    "explorer", "exported", "exposure", "extended", "external", "extracts",
    "fabulous", "facebook", "facility", "factored", "failures", "faithful",
    "familiar", "families", "fantastic", "fashions", "featured", "features",
    "feedback", "feelings", "festival", "fictional", "fighters", "fighting",
    "filename", "filtered", "finalist", "finances", "findings", "finished",
    "firearms", "firewood", "flagship", "floating", "flooding", "focusing",
    "followed", "follower", "football", "forecast", "forehead", "foremost",
    "formerly", "formulas", "fortress", "forwards", "founding", "fountain",
    "fraction", "fragment", "framing", "franklin", "freedoms", "frequent",
    "friendly", "frontier", "frontier", "function", "funding", "funerals",
    "furniture", "gambling", "gameplay", "gardener", "gathered", "generate",
    "generous", "genetics", "genocide", "gentlemen", "geography", "geometry",
    "girlfriend", "glorious", "gorgeous", "governed", "governor", "graceful",
    "graduate", "graphics", "grateful", "greatest", "greeting", "gripping",
    "grooming", "grounded", "grouping", "guarantee", "guardian", "guidance",
    "guideline", "guitarist", "habitats", "hallmark", "handbook", "handsome",
    "happened", "happiest", "happily", "hardware", "harmless", "harmony",
    "harshest", "hashable", "hastened", "headline", "headache", "healings",
    "healthier", "heartfelt", "heavenly", "hedgehog", "heighten", "helicopter",
    "helpless", "heritage", "hesitant", "highland", "highways", "hilarious",
    "historic", "hobbyist", "holdings", "holidays", "homeland", "homeless",
    "homepage", "homework", "honoring", "hopeless", "horizons", "horrible",
    "hospital", "hostname", "hostages", "hotlines", "household", "hundreds",
    "hydrogen", "hypnotic", "idealism", "identify", "identity", "ideology",
    "ignorant", "ignoring", "imagined", "imagines", "immature", "immersed",
    "immunity", "impacted", "imperial", "implicit", "imported", "imposing",
    "improved", "inactive", "incident", "included", "includes", "incoming",
    "increase", "indicate", "indirect", "industry", "infamous", "infected",
    "infinite", "informal", "informed", "inherent", "injected", "innocent",
    "inspired", "instance", "instinct", "integral", "intended", "interact",
    "interest", "interior", "internal", "internet", "interval", "intimate",
    "invented", "inventor", "invested", "investor", "involved", "involves",
    "isolated", "jealousy", "jeopardy", "journals", "judgment", "junction",
    "keyboard", "keywords", "kindness", "kingdoms", "knockout", "labeling",
    "landmark", "language", "launched", "launcher", "lawsuits", "layering",
    "layouts", "laziness", "leadersh", "learning", "lecturer", "leftover",
    "legacies", "legalese", "legends", "lemonade", "lengthen", "lessened",
    "leverage", "licensed", "lifelike", "lifetime", "lighting", "likewise",
    "limerick", "limiting", "linkedin", "listings", "literacy", "literary",
    "litigant", "liveable", "livelier", "loadings", "lobbyist", "locating",
    "location", "lockdown", "logistic", "lonesome", "longtime", "loophole",
    "lotteries", "luckiest", "luminous", "luncheon", "magnetic", "maintain",
    "majority", "makeover", "managing", "mandates", "manifest", "mansions",
    "marathon", "marginal", "maritime", "marketed", "marketer", "marriage",
    "massacre", "massaged", "mastered", "matching", "material", "maximize",
    "meanings", "measured", "mechanic", "medieval", "meetings", "melodies",
    "membrane", "memorial", "memories", "merchant", "merciful", "messages",
    "metadata", "metaphor", "military", "minimize", "minister", "minority",
    "miracles", "mischief", "misguide", "missions", "mistakes", "mixtures",
    "mobility", "modeling", "moderate", "modified", "momentum", "monetary",
    "monitors", "monolith", "monsters", "monument", "moonrise", "morality",
    "moreover", "mortgage", "mosquito", "motivated", "mountain", "mounting",
    "mourning", "movement", "multiple", "murdered", "murderer", "musician",
    "mutation", "mystical", "national", "navigate", "nearness", "neatness",
    "negative", "neglects", "neighbor", "networks", "neutered", "newborns",
    "newcomer", "nickname", "nineteen", "nobility", "nodetype", "nominate",
    "nonsense", "normally", "northern", "notably", "notebook", "noticing",
    "nowadays", "nuisance", "numbered", "numerous", "nurtured", "nutrient",
    "objected", "obtained", "occasion", "occupied", "occurred", "offering",
    "officers", "official", "offshore", "olympics", "omitting", "oncoming",
    "openings", "operated", "operator", "opinions", "opponent", "opposing",
    "opposite", "optimism", "optional", "ordering", "ordinary", "organism",
    "organize", "oriented", "original", "orphaned", "orthodox", "outbreak",
    "outcomes", "outdated", "outdoors", "outlined", "outraged", "outreach",
    "outsider", "overcome", "overhead", "overlaps", "overlook", "overseas",
    "overtime", "overview", "packages", "packaged", "painting", "pandemic",
    "panorama", "papercut", "paradigm", "paradise", "parallel", "paranoid",
    "parental", "parkland", "parlance", "parodies", "partakes", "particle",
    "partisan", "partners", "passages", "passions", "passport", "password",
    "patience", "patients", "patented", "patterns", "pavement", "payments",
    "peaceful", "pedagogy", "pedaling", "peculiar", "penitent", "pensions",
    "pentagon", "perceive", "performs", "periodic", "perished", "persists",
    "personal", "persuade", "pertains", "petition", "pharmacy", "pheasant",
    "phillips", "phonetic", "physical", "pictured", "pictures", "pilgrims",
    "pinpoint", "pipeline", "pitfalls", "placards", "placings", "plaguing",
    "plainest", "plaintif", "planning", "planted", "plastics", "platform",
    "plaudits", "playable", "playback", "playbook", "playlist", "pleasant",
    "pleasure", "pledging", "plethora", "plotting", "plunging", "podcasts",
    "poignant", "pointing", "polarize", "policies", "polished", "politely",
    "politics", "polluted", "ponderer", "populace", "populate", "portable",
    "portrait", "portugal", "position", "positive", "possible", "possibly",
    "postcard", "postmark", "postpone", "potatoes", "potently", "poultice",
    "pounding", "powerful", "practice", "praising", "preceded", "precious",
    "preclude", "predator", "predicts", "preempts", "prefaced", "prefaces",
    "prefixes", "pregnant", "prejudge", "premised", "premises", "premiums",
    "prenatal", "prepared", "presence", "preserve", "presided", "pressing",
    "pressure", "prestige", "presumed", "pretends", "prettify", "prettier",
    "previous", "pricecut", "primeval", "princess", "printing", "priority",
    "prisoner", "pristine", "probably", "problems", "proceeds", "produced",
    "producer", "products", "profiles", "profound", "programs", "progress",
    "projects", "prolific", "prologue", "promised", "promotes", "prompted",
    "promptly", "pronouns", "properly", "property", "prophecy", "proposal",
    "proposed", "prospect", "prospers", "protects", "proteins", "protocol",
    "proudest", "provider", "provides", "province", "provoked", "prowlers",
    "prudence", "publicly", "puddling", "punished", "purchase", "pursuing",
    "pyramids", "quadrant", "qualifies", "quantity", "quarters", "question",
    "quickest", "quietest", "quotient", "radiates", "radicals", "railroad",
    "rainfull", "randomly", "rankings", "ratified", "rational", "reacting",
    "reaction", "readable", "readying", "realigns", "realized", "realizes",
    "reappear", "rearmost", "rearward", "reasoned", "reassure", "rebelled",
    "rebooted", "rebuffed", "recalled", "receipts", "received", "receiver",
    "recently", "recharge", "reckless", "reckoned", "reclaims", "recorded",
    "recovers", "recovery", "recreate", "recruits", "redirect", "reducing",
    "referees", "referred", "reflects", "reformed", "reformer", "refugees",
    "refusing", "regarded", "regiment", "regional", "register", "regulate",
    "rehearse", "rejected", "relating", "relation", "relative", "relaxing",
    "released", "relevant", "reliable", "relieved", "religion", "reloaded",
    "relocate", "remained", "remaking", "remedial", "remedied", "remember",
    "reminded", "remixing", "remotely", "removals", "removing", "rendered",
    "renewing", "renowned", "repartee", "repeated", "replaced", "replicas",
    "replying", "reported", "reporter", "reposted", "reprints", "reproach",
    "republic", "requests", "required", "requires", "rerouted", "research",
    "resemble", "reserved", "resetter", "resident", "resigned", "resisted",
    "resolved", "resonant", "resorted", "resource", "responds", "response",
    "restarts", "restless", "restored", "restrict", "resulted", "retailed",
    "retained", "retirees", "retiring", "retraced", "retreats", "returned",
    "revealed", "reveling", "revenues", "reverses", "reviewed", "reviewer",
    "revising", "revision", "revolved", "rewarded", "reworded", "rhetoric",
    "richness", "ridicule", "rigidity", "rigorous", "rippling", "roadkill",
    "roadside", "robotics", "rocketed", "rollback", "romantic", "rosebush",
    "rotating", "rotation", "roughest", "roundest", "routines", "royalist",
    "rubbings", "rudeness", "ruefully", "ruffling", "ruggedly", "rumbling",
    "runabout", "rundowns", "runnings", "ruptured", "ruthless", "sabotage",
    "saddened", "safeness", "sailboat", "salaried", "sampling", "sanction",
    "sandwich", "sanitary", "saplings", "sarcastic", "saturday", "sausages",
    "savagely", "saveings", "scandals", "scanning", "scarcely", "scarcity",
    "scenario", "schedule", "scheming", "scholars", "sciences", "scissors",
    "scooping", "scouring", "scramble", "scrapers", "screamed", "screener",
    "scripted", "scrolled", "scrutiny", "sculptor", "seamless", "searched",
    "searcher", "seasonal", "secluded", "secondly", "sections", "securing",
    "security", "seedling", "segments", "selected", "senators", "sensible",
    "separate", "sequence", "sergeant", "servants", "services", "sessions",
    "settings", "settlers", "settling", "severely", "sexually", "shadowed",
    "shallows", "shambles", "shameful", "shielded", "shifting", "shilling",
    "shipping", "shoeless", "shooters", "shooting", "shopping", "shortage",
    "shortest", "shoulder", "shouting", "showcase", "showdown", "showered",
    "showings", "shutdown", "siblings", "sickness", "sidewalk", "sideways",
    "signaled", "silenced", "silently", "simplest", "simplify", "sincerly",
    "singular", "situated", "sketched", "skillets", "skipping", "slamming",
    "slapping", "slashing", "sleeping", "slightly", "slipping", "slippery",
    "sloggers", "slowdown", "smallest", "smashing", "smelling", "smoother",
    "smoothly", "smuggled", "snapshot", "sneaking", "snowball", "snowfall",
    "socially", "societal", "software", "soldiers", "solitary", "solution",
    "somebody", "somebody", "somedays", "sometime", "somewhat", "soothing",
    "sorority", "sounding", "southern", "souvenir", "spacious", "spanking",
    "spanning", "sparking", "speakers", "speaking", "specials", "specific",
    "specimen", "specters", "spectrum", "speeches", "speedway", "spelling",
    "spending", "spinning", "spirited", "splashed", "splendid", "splendor",
    "splinter", "splitting", "sponsors", "sporting", "spotless", "spotlight",
    "sprawled", "spraying", "spreader", "sprinkle", "sprinted", "squarely",
    "squashed", "squeaked", "squeezed", "stabbing", "stacking", "staffing",
    "stagnant", "staining", "stalking", "stallion", "stamping", "standard",
    "standing", "standout", "starling", "starting", "startled", "stashing",
    "statemen", "stations", "steadily", "steaming", "steering", "stepping",
    "sticking", "stiffest", "stifling", "stimulus", "stinging", "stirring",
    "stitched", "stockers", "stocking", "stomping", "stopping", "storages",
    "storming", "straddle", "straight", "strained", "stranded", "stranger",
    "strategy", "streamed", "strength", "stressed", "stretchy", "strictly",
    "striking", "stripped", "striving", "strolled", "stronger", "strongly",
    "struggle", "stubborn", "students", "studying", "stumbled", "stunning",
    "subjects", "submerge", "submited", "suburban", "succeeds", "succinct",
    "suddenly", "suffered", "suggests", "suitable", "suitably", "summoned",
    "sunlight", "sunshine", "superior", "supplied", "supplier", "supplies",
    "supports", "supposed", "suppress", "surfaces", "surgical", "surnamed",
    "surround", "surveyed", "surveyor", "survival", "survived", "survivor",
    "suspects", "suspends", "suspense", "sustains", "sweeping", "sweetest",
    "switches", "symbolic", "symptoms", "syndrome", "tackling", "tactical",
    "tailored", "takeover", "talented", "tangible", "targeted", "teaching",
    "teammate", "teamwork", "teardown", "teardrop", "technica", "teenager",
    "telegram", "tellling", "template", "temporal", "tendency", "tensions",
    "terminal", "terrific", "terrible", "terribly", "territor", "textbook",
    "textures", "thankful", "theaters", "theistic", "theology", "theorems",
    "theories", "theorize", "therapal", "thinking", "thorough", "thoughts",
    "thousand", "threaten", "thrilled", "thriller", "thriving", "throwing",
    "thursday", "timeline", "tireless", "tolerant", "tolerate", "tonights",
    "topology", "torments", "tortured", "touching", "toughest", "tourists",
    "township", "tracking", "traction", "tradeoff", "trailing", "training",
    "trampled", "transfer", "tranquil", "transmit", "transpar", "traveled",
    "traveler", "treasure", "treating", "treaties", "trekking", "trembled",
    "trending", "triangle", "tributes", "trickery", "triggers", "trillion",
    "trimming", "tripling", "trivials", "trophies", "tropical", "troubled",
    "troubles", "trucking", "trueness", "trustees", "truthful", "tumbling",
    "turmoils", "turnover", "turnsout", "tutorial", "twisters", "twisting",
    "ultimate", "umbrella", "unbiased", "uncommon", "uncovers", "underdog",
    "underway", "unfairly", "unfolded", "unharmed", "unhelpfu", "uniforms",
    "uniquely", "universe", "unjustly", "unknown", "unlikely", "unlocked",
    "unmarked", "unmasked", "unneeded", "unpacked", "unsigned", "unstable",
    "unsuited", "untested", "upcoming", "updating", "upgraded", "upheaval",
    "uploaded", "upstairs", "upstream", "urgently", "utilized", "utilizes",
    "uttering", "vacation", "vaccines", "validity", "valuable", "vanished",
    "variants", "varities", "vastness", "vehicles", "ventures", "verified",
    "versions", "veterans", "vibrancy", "vicinity", "victoria", "victuals",
    "viewings", "villager", "villages", "violates", "violence", "virtuous",
    "visiable", "visiting", "visitors", "visually", "vitamins", "vocalist",
    "volcanic", "voltages", "voluntar", "vouchers", "waisting", "waitlist",
    "walkable", "walkways", "wandered", "wantoned", "wardenly", "warfares",
    "warnings", "warranty", "watchdog", "watching", "watering", "weakened",
    "weakness", "wealthly", "weaponry", "wearable", "websites", "weekdays",
    "weekends", "weighing", "weighted", "welcomed", "welcomes", "wellbein",
    "wellness", "westward", "whatever", "whenever", "wherever", "whistled",
    "whistler", "widening", "wildlife", "willingu", "windfull", "wireless",
    "withdraw", "withhold", "withheld", "withouts", "witneess", "wondered",
    "woodwork", "workshop", "worrying", "worsened", "worthies", "worthily",
    "wouldn't", "wounding", "wrapping", "wreckage", "wrecking", "wrestler",
    "writings", "wrongful", "yachting", "yawnings", "yearbook", "yearning",
    "yielding", "youngest", "yourself", "youthful", "zealotry", "zeppelin",
    // Words with uncommon letters (j, k, q, x, z) for coverage
    "jacket", "jacked", "jammer", "jarred", "jazzer", "jeered", "jigsaw",
    "jinxed", "jogged", "jogger", "joined", "joiner", "joints", "joking",
    "jolted", "jostke", "judged", "juiced", "jumped", "jumper", "jungle",
    "junior", "junked", "kicked", "kicker", "kidnap", "killed", "killer",
    "kindle", "kindly", "kissed", "kitten", "knacks", "kneads", "kneels",
    "knifed", "knight", "knives", "knocks", "knotty", "quirks", "quirky",
    "quorum", "quoted", "quotes", "taxing", "taxied", "exacts", "exalts",
    "exceed", "excels", "except", "excess", "excite", "excuse", "exempt",
    "exerts", "exiled", "exists", "exotic", "expand", "expect", "expert",
    "expire", "export", "expose", "extend", "extern", "extras", "fixing",
    "fixups", "foxier", "foxing", "hexing", "hoaxed", "hoaxer", "jinxes",
    "laxity", "luxury", "maxing", "maxout", "mixing", "mixups", "nexted",
    "oxford", "oxygen", "pixels", "pixies", "reflex", "relays", "taxied",
    "toxics", "vexing", "waxing", "zombie", "zoning", "buzzer", "buzzed",
    "frozen", "fizzle", "fizzes", "fizzed", "frazzl", "frenzy", "freeze",
    "fuzzer", "fuzzed", "glazed", "glazes", "grazed", "grazer", "hazard",
    "hazier", "hazily", "hazing", "jazzed", "jazzer", "jazzle", "muzzle",
    "nozzle", "puzzle", "pizzas", "prized", "prizes", "razzle", "razzes",
    "seized", "seizer", "seizes", "sizing", "sleaze", "sneeze", "snazzy",
    "sozied", "tezing", "wizard", "zeroed", "zeroes", "zested", "zigzag",
    "zipped", "zipper", "zodiac", "zombie", "zoomed", "zoomer",
];

/// Source of prompt text
#[derive(Debug, Clone, PartialEq)]
pub enum PromptSource {
    /// Use randomly generated prompts from embedded/file words
    Random,
    /// Use only words from external file (error if not present)
    File,
    /// Use manual PROMPT_TEXT env var (legacy mode)
    Manual,
}

impl Default for PromptSource {
    fn default() -> Self {
        PromptSource::Random
    }
}

/// Type of generated prompt
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptType {
    SingleWord,
    Phrase,
    Sentence,
    Manual,
}

impl PromptType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PromptType::SingleWord => "single_word",
            PromptType::Phrase => "phrase",
            PromptType::Sentence => "sentence",
            PromptType::Manual => "manual",
        }
    }
}

/// Metadata about a generated prompt
#[derive(Debug, Clone, Serialize)]
pub struct PromptMetadata {
    /// Type of prompt generated
    pub prompt_type: String,
    /// Source of the prompt
    pub prompt_source: String,
    /// Number of words in the prompt
    pub word_count: usize,
    /// Whether baseline guide was displayed
    pub baseline_visible: bool,
    /// Y-position of baseline (if visible), in canvas coordinates
    pub baseline_y: Option<u32>,
    /// Index of this prompt in the session (1-indexed)
    pub prompt_index: u32,
    /// Schema version for future compatibility
    pub schema_version: String,
    /// Canvas bounding box - x origin (left edge)
    pub canvas_x: u32,
    /// Canvas bounding box - y origin (top edge)
    pub canvas_y: u32,
    /// Canvas bounding box - width
    pub canvas_width: u32,
    /// Canvas bounding box - height
    pub canvas_height: u32,
}

impl Default for PromptMetadata {
    fn default() -> Self {
        Self {
            prompt_type: "manual".to_string(),
            prompt_source: "manual".to_string(),
            word_count: 0,
            baseline_visible: false,
            baseline_y: None,
            prompt_index: 1,
            schema_version: "2.0".to_string(),
            canvas_x: 0,
            canvas_y: 0,
            canvas_width: 0,
            canvas_height: 0,
        }
    }
}

/// Configuration for prompt generation
#[derive(Debug, Clone)]
pub struct PromptConfig {
    /// Source of prompts
    pub source: PromptSource,
    /// Path to optional word file override
    pub word_file_override: Option<PathBuf>,
    /// Weight for single word prompts (0-100)
    pub weight_single: u8,
    /// Weight for phrase prompts (0-100)
    pub weight_phrase: u8,
    /// Weight for sentence prompts (0-100)
    pub weight_sentence: u8,
    /// Range of words in phrases (min, max)
    pub phrase_words: (usize, usize),
    /// Range of words in sentences (min, max)
    pub sentence_words: (usize, usize),
    /// Whether baseline guide is enabled
    pub baseline_enabled: bool,
    /// Y offset of baseline from canvas bottom
    pub baseline_y_offset: u32,
    /// Optional random seed for reproducibility
    pub seed: Option<u64>,
    /// Canvas bounding box (x, y, width, height) for stroke normalization
    pub canvas_bounds: (u32, u32, u32, u32),
}

impl Default for PromptConfig {
    fn default() -> Self {
        Self {
            source: PromptSource::Random,
            word_file_override: None,
            weight_single: 40,
            weight_phrase: 40,
            weight_sentence: 20,
            phrase_words: (2, 5),
            sentence_words: (3, 8),
            baseline_enabled: true,
            baseline_y_offset: 100,
            seed: None,
            canvas_bounds: (0, 0, 0, 0),
        }
    }
}

impl PromptConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Parse source
        if let Ok(source) = env::var("PROMPT_SOURCE") {
            config.source = match source.to_lowercase().as_str() {
                "file" => PromptSource::File,
                "manual" => PromptSource::Manual,
                _ => PromptSource::Random,
            };
        }

        // Parse word file path
        if let Ok(path) = env::var("WORDS_FILE") {
            config.word_file_override = Some(PathBuf::from(path));
        }

        // Parse weights
        if let Ok(w) = env::var("PROMPT_WEIGHT_SINGLE") {
            config.weight_single = w.parse().unwrap_or(40);
        }
        if let Ok(w) = env::var("PROMPT_WEIGHT_PHRASE") {
            config.weight_phrase = w.parse().unwrap_or(40);
        }
        if let Ok(w) = env::var("PROMPT_WEIGHT_SENTENCE") {
            config.weight_sentence = w.parse().unwrap_or(20);
        }

        // Parse word count ranges
        if let Ok(min) = env::var("PHRASE_MIN_WORDS") {
            config.phrase_words.0 = min.parse().unwrap_or(2);
        }
        if let Ok(max) = env::var("PHRASE_MAX_WORDS") {
            config.phrase_words.1 = max.parse().unwrap_or(5);
        }
        if let Ok(min) = env::var("SENTENCE_MIN_WORDS") {
            config.sentence_words.0 = min.parse().unwrap_or(3);
        }
        if let Ok(max) = env::var("SENTENCE_MAX_WORDS") {
            config.sentence_words.1 = max.parse().unwrap_or(8);
        }

        // Parse baseline settings
        if let Ok(enabled) = env::var("BASELINE_ENABLED") {
            config.baseline_enabled = enabled.to_lowercase() == "true";
        }
        if let Ok(offset) = env::var("BASELINE_Y_OFFSET") {
            config.baseline_y_offset = offset.parse().unwrap_or(100);
        }

        // Parse seed
        if let Ok(seed) = env::var("RANDOM_SEED") {
            config.seed = seed.parse().ok();
        }

        config
    }
}

/// A generated prompt with text and metadata
#[derive(Debug, Clone)]
pub struct GeneratedPrompt {
    /// The prompt text to display
    pub text: String,
    /// Type of prompt
    pub prompt_type: PromptType,
    /// Number of words
    pub word_count: usize,
    /// Full metadata
    pub metadata: PromptMetadata,
}

/// Prompt generator that produces varied handwriting prompts
pub struct PromptGenerator {
    config: PromptConfig,
    words: Vec<String>,
    rng: StdRng,
    prompt_count: u32,
}

impl PromptGenerator {
    /// Create a new generator with the given configuration
    pub fn new(config: PromptConfig) -> Self {
        // Initialize RNG
        let rng = match config.seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };

        // Load words
        let words = Self::load_words(&config);

        Self {
            config,
            words,
            rng,
            prompt_count: 0,
        }
    }

    /// Create generator from environment variables
    pub fn from_env() -> Self {
        Self::new(PromptConfig::from_env())
    }

    /// Load words from embedded list and/or file
    fn load_words(config: &PromptConfig) -> Vec<String> {
        let mut words: Vec<String> = Vec::new();

        // Load from file if specified
        if let Some(ref path) = config.word_file_override {
            if let Ok(contents) = fs::read_to_string(path) {
                for line in contents.lines() {
                    let line = line.trim();
                    // Skip comments and empty lines
                    if !line.is_empty() && !line.starts_with('#') {
                        words.push(line.to_string());
                    }
                }
                println!("Loaded {} words from {:?}", words.len(), path);
            } else {
                eprintln!("Warning: Could not read word file {:?}", path);
            }
        }

        // If File-only mode and no words loaded, that's an error
        if config.source == PromptSource::File && words.is_empty() {
            eprintln!("Error: PROMPT_SOURCE=file but no words loaded from file");
        }

        // Add embedded words unless File-only mode
        if config.source != PromptSource::File {
            // Add embedded words (avoiding duplicates)
            let existing: std::collections::HashSet<_> = words.iter().cloned().collect();
            for word in EMBEDDED_WORDS {
                if !existing.contains(*word) {
                    words.push(word.to_string());
                }
            }
        }

        if words.is_empty() {
            // Fallback to embedded words
            words = EMBEDDED_WORDS.iter().map(|s| s.to_string()).collect();
        }

        words
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &PromptConfig {
        &self.config
    }

    /// Get the current prompt count
    pub fn count(&self) -> u32 {
        self.prompt_count
    }

    /// Generate the next prompt
    pub fn next(&mut self) -> GeneratedPrompt {
        self.prompt_count += 1;

        // Handle manual mode
        if self.config.source == PromptSource::Manual {
            let text = env::var("PROMPT_TEXT")
                .unwrap_or_else(|_| "Write something below".to_string());
            let word_count = text.split_whitespace().count();
            return GeneratedPrompt {
                text: text.clone(),
                prompt_type: PromptType::Manual,
                word_count,
                metadata: PromptMetadata {
                    prompt_type: "manual".to_string(),
                    prompt_source: "manual".to_string(),
                    word_count,
                    baseline_visible: self.config.baseline_enabled,
                    baseline_y: if self.config.baseline_enabled {
                        Some(self.config.baseline_y_offset)
                    } else {
                        None
                    },
                    prompt_index: self.prompt_count,
                    schema_version: "2.0".to_string(),
                    canvas_x: self.config.canvas_bounds.0,
                    canvas_y: self.config.canvas_bounds.1,
                    canvas_width: self.config.canvas_bounds.2,
                    canvas_height: self.config.canvas_bounds.3,
                },
            };
        }

        // Select prompt type based on weights
        let prompt_type = self.select_prompt_type();

        // Generate prompt based on type
        let (text, word_count) = match prompt_type {
            PromptType::SingleWord => self.generate_single_word(),
            PromptType::Phrase => self.generate_phrase(),
            PromptType::Sentence => self.generate_sentence(),
            PromptType::Manual => unreachable!(),
        };

        let prompt_source = if self.config.word_file_override.is_some() {
            "random_file"
        } else {
            "random_embedded"
        };

        GeneratedPrompt {
            text,
            prompt_type,
            word_count,
            metadata: PromptMetadata {
                prompt_type: prompt_type.as_str().to_string(),
                prompt_source: prompt_source.to_string(),
                word_count,
                baseline_visible: self.config.baseline_enabled,
                baseline_y: if self.config.baseline_enabled {
                    Some(self.config.baseline_y_offset)
                } else {
                    None
                },
                prompt_index: self.prompt_count,
                schema_version: "2.0".to_string(),
                canvas_x: self.config.canvas_bounds.0,
                canvas_y: self.config.canvas_bounds.1,
                canvas_width: self.config.canvas_bounds.2,
                canvas_height: self.config.canvas_bounds.3,
            },
        }
    }

    /// Select a prompt type based on configured weights
    fn select_prompt_type(&mut self) -> PromptType {
        let total = self.config.weight_single as u32
            + self.config.weight_phrase as u32
            + self.config.weight_sentence as u32;

        if total == 0 {
            return PromptType::SingleWord;
        }

        let roll: u32 = self.rng.gen_range(0..total);

        if roll < self.config.weight_single as u32 {
            PromptType::SingleWord
        } else if roll < (self.config.weight_single + self.config.weight_phrase) as u32 {
            PromptType::Phrase
        } else {
            PromptType::Sentence
        }
    }

    /// Generate a single word prompt
    fn generate_single_word(&mut self) -> (String, usize) {
        let word = self.random_word();
        (word, 1)
    }

    /// Generate a phrase (2-5 words, optional light punctuation)
    fn generate_phrase(&mut self) -> (String, usize) {
        let (min, max) = self.config.phrase_words;
        let count = self.rng.gen_range(min..=max);

        let words: Vec<String> = (0..count).map(|_| self.random_word()).collect();

        // 50% chance to capitalize first word
        let mut result = words.clone();
        if self.rng.gen_bool(0.5) {
            result[0] = capitalize(&result[0]);
        }

        // 20% chance of comma if 4+ words
        let text = if count >= 4 && self.rng.gen_bool(0.2) {
            let comma_pos = self.rng.gen_range(1..count - 1);
            let mut parts: Vec<String> = result.clone();
            parts[comma_pos] = format!("{},", parts[comma_pos]);
            parts.join(" ")
        } else {
            result.join(" ")
        };

        (text, count)
    }

    /// Generate a sentence (3-8 words with punctuation)
    fn generate_sentence(&mut self) -> (String, usize) {
        let (min, max) = self.config.sentence_words;
        let count = self.rng.gen_range(min..=max);

        let words: Vec<String> = (0..count).map(|_| self.random_word()).collect();

        // Always capitalize first word
        let mut result = words.clone();
        result[0] = capitalize(&result[0]);

        // 30% chance of mid-sentence comma if 5+ words
        if count >= 5 && self.rng.gen_bool(0.3) {
            let comma_pos = self.rng.gen_range(1..count - 2);
            result[comma_pos] = format!("{},", result[comma_pos]);
        }

        // 10% chance of possessive
        if self.rng.gen_bool(0.1) && count >= 2 {
            let pos = self.rng.gen_range(0..count - 1);
            result[pos] = format!("{}'s", result[pos]);
        }

        let mut text = result.join(" ");

        // Select ending punctuation: 70% period, 15% exclamation, 15% question
        let ending_roll: f64 = self.rng.gen();
        let ending = if ending_roll < 0.70 {
            "."
        } else if ending_roll < 0.85 {
            "!"
        } else {
            "?"
        };

        text.push_str(ending);

        (text, count)
    }

    /// Get a random word from the word list
    fn random_word(&mut self) -> String {
        if self.words.is_empty() {
            return "word".to_string();
        }
        let idx = self.rng.gen_range(0..self.words.len());
        self.words[idx].clone()
    }

    /// Reset the generator with optional new seed
    pub fn reset(&mut self, seed: Option<u64>) {
        self.prompt_count = 0;
        self.rng = match seed.or(self.config.seed) {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_entropy(),
        };
    }

    /// Set the canvas bounds for stroke normalization metadata
    pub fn set_canvas_bounds(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.config.canvas_bounds = (x, y, width, height);
    }
}

/// Capitalize the first letter of a string
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PromptConfig::default();
        assert_eq!(config.weight_single, 40);
        assert_eq!(config.weight_phrase, 40);
        assert_eq!(config.weight_sentence, 20);
        assert!(config.baseline_enabled);
    }

    #[test]
    fn test_generator_with_seed() {
        let config = PromptConfig {
            seed: Some(12345),
            ..Default::default()
        };
        let mut gen1 = PromptGenerator::new(config.clone());
        let mut gen2 = PromptGenerator::new(config);

        // Same seed should produce same results
        let p1 = gen1.next();
        let p2 = gen2.next();
        assert_eq!(p1.text, p2.text);
    }

    #[test]
    fn test_single_word_no_punctuation() {
        let config = PromptConfig {
            weight_single: 100,
            weight_phrase: 0,
            weight_sentence: 0,
            seed: Some(42),
            ..Default::default()
        };
        let mut gen = PromptGenerator::new(config);
        let prompt = gen.next();

        assert_eq!(prompt.prompt_type, PromptType::SingleWord);
        assert_eq!(prompt.word_count, 1);
        // Single words should not have punctuation
        assert!(!prompt.text.contains('.'));
        assert!(!prompt.text.contains(','));
        assert!(!prompt.text.contains('!'));
        assert!(!prompt.text.contains('?'));
    }

    #[test]
    fn test_sentence_has_ending() {
        let config = PromptConfig {
            weight_single: 0,
            weight_phrase: 0,
            weight_sentence: 100,
            seed: Some(42),
            ..Default::default()
        };
        let mut gen = PromptGenerator::new(config);
        let prompt = gen.next();

        assert_eq!(prompt.prompt_type, PromptType::Sentence);
        // Sentences should end with . ! or ?
        let text = &prompt.text;
        assert!(
            text.ends_with('.') || text.ends_with('!') || text.ends_with('?'),
            "Sentence should end with punctuation: {}",
            text
        );
    }

    #[test]
    fn test_prompt_count_increments() {
        let mut gen = PromptGenerator::new(PromptConfig::default());
        assert_eq!(gen.count(), 0);

        gen.next();
        assert_eq!(gen.count(), 1);

        gen.next();
        assert_eq!(gen.count(), 2);
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("hello"), "Hello");
        assert_eq!(capitalize("WORLD"), "WORLD");
        assert_eq!(capitalize(""), "");
        assert_eq!(capitalize("a"), "A");
    }
}
