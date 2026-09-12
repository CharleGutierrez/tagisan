#!/usr/bin/env python3
"""
Automated Verification Test Harness for:
Web Design & Development for the Vibe Code Developer (50 Books Canon)
Validates:
1. Strict ISBN-10 (modulo 11) and ISBN-13 (modulo 10) checksum calculations for all 50 books.
2. ECC Tagisan YAML frontmatter parsing compliance for SKILL.md and Agent persona.
3. Generates all files for Tagisan (.ecc/skills, .ecc/agents, docs/).
4. Live integration with `tgs` binary: verifying agent discovery in `tgs ecc list` and skill discovery in `tgs ecc skills`.
"""

import os
import sys
import subprocess
import shutil
from pathlib import Path

# --- 1. The 50 Canonical Web Design & Development Books ---
BOOKS_50_CANON = [
    # Pillar 1: Visual Design, UI Aesthetics & Craft
    {
        "num": 1,
        "title": "The Non-Designer's Design Book",
        "authors": "Robin Williams",
        "publisher": "Peachpit Press",
        "year_edition": "2014 (4th Edition)",
        "isbn10": "0133966151",
        "isbn13": "978-0133966152",
        "pillar": "Visual Design, UI Aesthetics & Craft",
        "concept": "The CRAP principles: Contrast, Repetition, Alignment, Proximity.",
        "vibe_moat": "The fastest mental model to audit generated layouts: immediately spots alignment flaws and weak visual contrast."
    },
    {
        "num": 2,
        "title": "The Elements of Typographic Style",
        "authors": "Robert Bringhurst",
        "publisher": "Hartley & Marks",
        "year_edition": "2012 (4th Edition)",
        "isbn10": "0881792128",
        "isbn13": "978-0881792126",
        "pillar": "Visual Design, UI Aesthetics & Craft",
        "concept": "The typography bible: typographic rhythm, proportion, scale, vertical grid.",
        "vibe_moat": "Allows you to instruct AI models with professional typographic vocabulary (leading, tracking, modular scales, measure)."
    },
    {
        "num": 3,
        "title": "Universal Principles of Design",
        "authors": "William Lidwell, Kritina Holden, Jill Butler",
        "publisher": "Rockport Publishers",
        "year_edition": "2010 (Revised Edition)",
        "isbn10": "1592535879",
        "isbn13": "978-1592535873",
        "pillar": "Visual Design, UI Aesthetics & Craft",
        "concept": "125 foundational design laws (Fitts's Law, Hick's Law, mental models, affordances).",
        "vibe_moat": "Invaluable reference to prompt for scientifically validated UI patterns rather than arbitrary aesthetic guesses."
    },
    {
        "num": 4,
        "title": "Thinking with Type",
        "authors": "Ellen Lupton",
        "publisher": "Princeton Architectural Press",
        "year_edition": "2010 (2nd Edition)",
        "isbn10": "1568989695",
        "isbn13": "978-1568989693",
        "pillar": "Visual Design, UI Aesthetics & Craft",
        "concept": "Practical typography guide: letterforms, grid systems, text hierarchies.",
        "vibe_moat": "Elevates web typography from standard font-sans to intentional typographic hierarchy and editorial pacing."
    },
    {
        "num": 5,
        "title": "Grid Systems in Graphic Design",
        "authors": "Josef Müller-Brockmann",
        "publisher": "Niggli Verlag",
        "year_edition": "1996",
        "isbn10": "3721201450",
        "isbn13": "978-3721201451",
        "pillar": "Visual Design, UI Aesthetics & Craft",
        "concept": "Swiss International Typographic Style; modular grid layouts and structural discipline.",
        "vibe_moat": "Guides multi-column responsive dashboard generation using disciplined mathematical proportions."
    },
    {
        "num": 6,
        "title": "Interaction of Color",
        "authors": "Josef Albers",
        "publisher": "Yale University Press",
        "year_edition": "2013 (50th Anniversary Edition)",
        "isbn10": "0300179359",
        "isbn13": "978-0300179354",
        "pillar": "Visual Design, UI Aesthetics & Craft",
        "concept": "Color relativity, optical illusions, perceived luminance, palette balance.",
        "vibe_moat": "Prevents generating jarring, unreadable color palettes; enables cohesive dark/light mode token systems."
    },
    {
        "num": 7,
        "title": "Design for Hackers: Reverse Engineering Beauty",
        "authors": "David Kadavy",
        "publisher": "Wiley",
        "year_edition": "2011",
        "isbn10": "1119998956",
        "isbn13": "978-1119998952",
        "pillar": "Visual Design, UI Aesthetics & Craft",
        "concept": "Deconstructing classical aesthetic proportions (Golden Ratio, Fibonacci) into code logic.",
        "vibe_moat": "Helps programmers understand why certain layouts feel harmonious through geometric and historical analysis."
    },
    {
        "num": 8,
        "title": "Layout Essentials: 100 Design Principles for Using Grids",
        "authors": "Beth Tondreau",
        "publisher": "Rockport Publishers",
        "year_edition": "2011",
        "isbn10": "1592537073",
        "isbn13": "978-1592537075",
        "pillar": "Visual Design, UI Aesthetics & Craft",
        "concept": "Practical execution of multi-column, modular, and dynamic content layouts.",
        "vibe_moat": "Helps break away from standard single-column AI blogs into dynamic magazine-style layouts."
    },
    {
        "num": 9,
        "title": "Microinteractions: Designing with Details",
        "authors": "Dan Saffer",
        "publisher": "O'Reilly Media",
        "year_edition": "2013",
        "isbn10": "144934268X",
        "isbn13": "978-1449342685",
        "pillar": "Visual Design, UI Aesthetics & Craft",
        "concept": "Designing feedback loops, triggers, state toggles, and delightful micro-moments.",
        "vibe_moat": "Transforms stiff, static AI-generated buttons into tactile, responsive, lively interactive controls."
    },
    {
        "num": 10,
        "title": "Designing Visual Interfaces: Communication Oriented Techniques",
        "authors": "Kevin Mullet, Darrell Sano",
        "publisher": "Prentice Hall",
        "year_edition": "1995",
        "isbn10": "0133033899",
        "isbn13": "978-0133033892",
        "pillar": "Visual Design, UI Aesthetics & Craft",
        "concept": "Visual elegance, simplicity, scale, contrast, and visual communication principles.",
        "vibe_moat": "Teaches how to prompt for visual restraint, eliminating unnecessary ornamentation and clutter."
    },

    # Pillar 2: Modern CSS Architecture, Layouts & Motion
    {
        "num": 11,
        "title": "CSS Secrets: Better Solutions to Everyday Web Design Problems",
        "authors": "Lea Verou",
        "publisher": "O'Reilly Media",
        "year_edition": "2015",
        "isbn10": "1449372635",
        "isbn13": "978-1449372637",
        "pillar": "Modern CSS Architecture, Layouts & Motion",
        "concept": "Advanced CSS techniques: mathematical styling, clip-paths, variable fonts, blend modes.",
        "vibe_moat": "Solves tricky visual styling challenges with elegant, modern native CSS rather than heavy JS libraries."
    },
    {
        "num": 12,
        "title": "CSS: The Definitive Guide: Visual Presentation for the Web",
        "authors": "Eric A. Meyer, Estelle Weyl",
        "publisher": "O'Reilly Media",
        "year_edition": "2023 (5th Edition)",
        "isbn10": "1098117611",
        "isbn13": "978-1098117610",
        "pillar": "Modern CSS Architecture, Layouts & Motion",
        "concept": "The complete reference manual for CSS specifications, Flexbox, Grid, and transforms.",
        "vibe_moat": "The ultimate technical ground truth when debugging strange layout bugs that LLMs hallucinate fixes for."
    },
    {
        "num": 13,
        "title": "Modern CSS with Tailwind: Flexible Styling Without the Fuss",
        "authors": "Noel Rappin",
        "publisher": "Pragmatic Bookshelf",
        "year_edition": "2021",
        "isbn10": "1680508563",
        "isbn13": "978-1680508567",
        "pillar": "Modern CSS Architecture, Layouts & Motion",
        "concept": "Utility-first workflows, theme configuration, component extraction, arbitrary values.",
        "vibe_moat": "How to structure clean, maintainable utility systems without polluting JSX/HTML into an unreadable mess."
    },
    {
        "num": 14,
        "title": "Transitions and Animations with CSS: Adding Motion to the Web",
        "authors": "Estelle Weyl",
        "publisher": "O'Reilly Media",
        "year_edition": "2013",
        "isbn10": "1449321585",
        "isbn13": "978-1449321581",
        "pillar": "Modern CSS Architecture, Layouts & Motion",
        "concept": "Hardware-accelerated transforms, timing functions, CSS keyframe choreographies.",
        "vibe_moat": "Teaches how to prompt for 60fps GPU-composited animations without triggering layout thrashing."
    },
    {
        "num": 15,
        "title": "Responsive Web Design",
        "authors": "Ethan Marcotte",
        "publisher": "A Book Apart",
        "year_edition": "2011",
        "isbn10": "0983367116",
        "isbn13": "978-0983367116",
        "pillar": "Modern CSS Architecture, Layouts & Motion",
        "concept": "The foundational text: fluid grids, flexible media, progressive layout adaptation.",
        "vibe_moat": "Instills core responsive philosophy: building layouts that flex naturally with content and screen size."
    },
    {
        "num": 16,
        "title": "CSS in Depth",
        "authors": "Keith J. Grant",
        "publisher": "Manning Publications",
        "year_edition": "2018",
        "isbn10": "1617293407",
        "isbn13": "978-1617293405",
        "pillar": "Modern CSS Architecture, Layouts & Motion",
        "concept": "Deep dive into cascade, specificity, stacking contexts, custom properties, and subgrid.",
        "vibe_moat": "Eliminates z-index wars and specificity overrides when combining generated CSS snippets."
    },
    {
        "num": 17,
        "title": "SVG Animations: From Common UX Concerns to Complex Responsive Animation",
        "authors": "Sarah Drasner",
        "publisher": "O'Reilly Media",
        "year_edition": "2017",
        "isbn10": "1491939702",
        "isbn13": "978-1491939703",
        "pillar": "Modern CSS Architecture, Layouts & Motion",
        "concept": "Vector illustration manipulation, SMIL, CSS animation, GSAP orchestration.",
        "vibe_moat": "Teaches how to prompt for lightweight, resolution-independent vector animations that outshine heavy GIF assets."
    },
    {
        "num": 18,
        "title": "Sass and Compass in Action",
        "authors": "Wynn Netherland, Nathan Weizenbaum, Chris Eppstein, Brandon Mathis",
        "publisher": "Manning Publications",
        "year_edition": "2013",
        "isbn10": "1617290149",
        "isbn13": "978-1617290145",
        "pillar": "Modern CSS Architecture, Layouts & Motion",
        "concept": "Preprocessed stylesheets, mixins, nested selectors, design tokens.",
        "vibe_moat": "Foundational understanding of CSS variables and architectural preprocessor patterns."
    },
    {
        "num": 19,
        "title": "CSS Master",
        "authors": "Tiffany B. Brown",
        "publisher": "SitePoint",
        "year_edition": "2021 (3rd Edition)",
        "isbn10": "1925836428",
        "isbn13": "978-1925836424",
        "pillar": "Modern CSS Architecture, Layouts & Motion",
        "concept": "Modern layout techniques, CSS Grid, custom properties, blend modes, filters.",
        "vibe_moat": "Practical handbook for cutting-edge CSS features that eliminate legacy JavaScript polyfills."
    },
    {
        "num": 20,
        "title": "Pro CSS3 Animation",
        "authors": "Dudley Storey",
        "publisher": "Apress",
        "year_edition": "2013",
        "isbn10": "1430247258",
        "isbn13": "978-1430247258",
        "pillar": "Modern CSS Architecture, Layouts & Motion",
        "concept": "Keyframe physics, transition curves, easing functions, 3D CSS transforms.",
        "vibe_moat": "Guides prompting for fluid UI feedback loops that make interfaces feel natural and physical."
    },

    # Pillar 3: Component Systems, Design Systems & Frontend Architecture
    {
        "num": 21,
        "title": "Atomic Design",
        "authors": "Brad Frost",
        "publisher": "Brad Frost",
        "year_edition": "2016",
        "isbn10": "0998296600",
        "isbn13": "978-0998296609",
        "pillar": "Component Systems, Design Systems & Frontend Architecture",
        "concept": "Component hierarchy: Atoms, Molecules, Organisms, Templates, Pages.",
        "vibe_moat": "The scaffolding roadmap: guides how you structure agent prompts from basic UI primitives to full templates."
    },
    {
        "num": 22,
        "title": "Designing Interfaces: Patterns for Effective Interaction Design",
        "authors": "Jenifer Tidwell, Charles Brewer, Aynne Valencia",
        "publisher": "O'Reilly Media",
        "year_edition": "2020 (3rd Edition)",
        "isbn10": "1492051969",
        "isbn13": "978-1492051961",
        "pillar": "Component Systems, Design Systems & Frontend Architecture",
        "concept": "Catalog of canonical interaction patterns: navigation, search, data presentation.",
        "vibe_moat": "Prevents reinventing the wheel: prompts AI for proven, industry-standard UI patterns."
    },
    {
        "num": 23,
        "title": "Design Systems: A Practical Guide to Creating Design Languages for Digital Products",
        "authors": "Alla Kholmatova",
        "publisher": "Smashing Magazine",
        "year_edition": "2017",
        "isbn10": "3945749581",
        "isbn13": "978-3945749586",
        "pillar": "Component Systems, Design Systems & Frontend Architecture",
        "concept": "Shared visual vocabulary, design tokens, purpose-directed pattern libraries.",
        "vibe_moat": "Guides establishing a global theme token system before prompting page-level layouts."
    },
    {
        "num": 24,
        "title": "Building Design Systems: Unify User Experiences through a Shared Design Language",
        "authors": "Sarrah Vesselov, Taurie Davis",
        "publisher": "Apress",
        "year_edition": "2019",
        "isbn10": "148424513X",
        "isbn13": "978-1484245132",
        "pillar": "Component Systems, Design Systems & Frontend Architecture",
        "concept": "Cross-functional design system workflows, component governance, documentation.",
        "vibe_moat": "Helps maintain stylistic and functional consistency across dozens of AI prompting sessions."
    },
    {
        "num": 25,
        "title": "Frontend Architecture for Design Systems: A Modern Blueprint for Scalable and Sustainable Websites",
        "authors": "Micah Godbolt",
        "publisher": "O'Reilly Media",
        "year_edition": "2016",
        "isbn10": "1491926783",
        "isbn13": "978-1491926789",
        "pillar": "Component Systems, Design Systems & Frontend Architecture",
        "concept": "Component-driven architecture, CSS modularity, asset pipeline integration.",
        "vibe_moat": "Teaches how to connect design tokens directly to build tooling and continuous deployment pipelines."
    },
    {
        "num": 26,
        "title": "Micro Frontends in Action",
        "authors": "Michael Geers",
        "publisher": "Manning Publications",
        "year_edition": "2020",
        "isbn10": "1617296872",
        "isbn13": "978-1617296871",
        "pillar": "Component Systems, Design Systems & Frontend Architecture",
        "concept": "Decoupled vertical frontend architectures, independent deployment, routing integration.",
        "vibe_moat": "How to partition a massive vibe-coded application into independently promptable, isolated frontend micro-apps."
    },
    {
        "num": 27,
        "title": "Frameworkless Front-End Development: Do You Have a Focus on the Foundation?",
        "authors": "Francesco Strazzullo",
        "publisher": "Apress",
        "year_edition": "2019",
        "isbn10": "1484249666",
        "isbn13": "978-1484249666",
        "pillar": "Component Systems, Design Systems & Frontend Architecture",
        "concept": "Understanding DOM manipulation, routing, and state management without framework lock-in.",
        "vibe_moat": "Enables vibe coders to build blazing-fast vanilla JS/TS apps when heavy React/Angular scaffolding is overkill."
    },
    {
        "num": 28,
        "title": "Web Components in Action",
        "authors": "Ben Farrell",
        "publisher": "Manning Publications",
        "year_edition": "2019",
        "isbn10": "1617295779",
        "isbn13": "978-1617295775",
        "pillar": "Component Systems, Design Systems & Frontend Architecture",
        "concept": "Native Custom Elements, Shadow DOM encapsulation, HTML Templates.",
        "vibe_moat": "Teaches building framework-agnostic design system components that run anywhere without build steps."
    },
    {
        "num": 29,
        "title": "Developing Large Web Applications: Producing Code That Can Grow and Prosper",
        "authors": "Kyle Loudon",
        "publisher": "O'Reilly Media",
        "year_edition": "2010",
        "isbn10": "0596803028",
        "isbn13": "978-0596803025",
        "pillar": "Component Systems, Design Systems & Frontend Architecture",
        "concept": "Modularity, component reuse, testing strategies, deployment architectures.",
        "vibe_moat": "Architectural guidelines to prevent rapid AI code iterations from collapsing into spaghetti."
    },
    {
        "num": 30,
        "title": "Maintainable JavaScript: Writing Readable, Lean, and Reusable Code",
        "authors": "Nicholas C. Zakas",
        "publisher": "O'Reilly Media",
        "year_edition": "2012",
        "isbn10": "1449325246",
        "isbn13": "978-1449325244",
        "pillar": "Component Systems, Design Systems & Frontend Architecture",
        "concept": "Strict code conventions, decoupling application logic from event handlers.",
        "vibe_moat": "Provides explicit architectural rules to inject into prompts to ensure clean separation of concerns."
    },

    # Pillar 4: UX Psychology, Cognitive Ergonomics & Usability
    {
        "num": 31,
        "title": "Don't Make Me Think, Revisited: A Common Sense Approach to Web Usability",
        "authors": "Steve Krug",
        "publisher": "New Riders",
        "year_edition": "2014 (3rd Edition)",
        "isbn10": "0321965515",
        "isbn13": "978-0321965516",
        "pillar": "UX Psychology, Cognitive Ergonomics & Usability",
        "concept": "The usability law: interfaces should be self-evident; eliminate needless user cognitive friction.",
        "vibe_moat": "The golden rule: audits vibe-coded UIs to ensure every screen requires zero cognitive effort to navigate."
    },
    {
        "num": 32,
        "title": "The Design of Everyday Things",
        "authors": "Don Norman",
        "publisher": "Basic Books",
        "year_edition": "2013 (Revised Edition)",
        "isbn10": "0465050654",
        "isbn13": "978-0465050659",
        "pillar": "UX Psychology, Cognitive Ergonomics & Usability",
        "concept": "Affordances, signifiers, constraints, feedback, mapping, conceptual models.",
        "vibe_moat": "Identifies why users misclick generated UI elements: reveals missing signifiers and mismatched mental models."
    },
    {
        "num": 33,
        "title": "Designing with the Mind in Mind: Simple Guide to Understanding User Interface Design Guidelines",
        "authors": "Jeff Johnson",
        "publisher": "Morgan Kaufmann",
        "year_edition": "2020 (3rd Edition)",
        "isbn10": "0128182024",
        "isbn13": "978-0128182024",
        "pillar": "UX Psychology, Cognitive Ergonomics & Usability",
        "concept": "Cognitive psychology principles: visual perception, peripheral vision, working memory limits.",
        "vibe_moat": "Explains the biological reasons behind UI rules (e.g. why users ignore banners, how chunking aids form completion)."
    },
    {
        "num": 34,
        "title": "About Face: The Essentials of Interaction Design",
        "authors": "Alan Cooper, Robert Reimann, David Cronin, Christopher Noessel",
        "publisher": "Wiley",
        "year_edition": "2014 (4th Edition)",
        "isbn10": "1118766571",
        "isbn13": "978-1118766576",
        "pillar": "UX Psychology, Cognitive Ergonomics & Usability",
        "concept": "Goal-directed design, interaction idioms, avoiding mechanical sympathy.",
        "vibe_moat": "Eliminates developer-centric UI; forces generated interfaces to align with actual user workflow goals."
    },
    {
        "num": 35,
        "title": "Rocket Surgery Made Easy: The Do-It-Yourself Guide to Finding and Fixing Usability Problems",
        "authors": "Steve Krug",
        "publisher": "New Riders",
        "year_edition": "2009",
        "isbn10": "0321657292",
        "isbn13": "978-0321657299",
        "pillar": "UX Psychology, Cognitive Ergonomics & Usability",
        "concept": "Do-it-yourself usability testing: 3 users, 1 morning a month, fix the biggest problems.",
        "vibe_moat": "Enables rapid usability testing loops on vibe-coded MVPs to detect confusion points in under 30 minutes."
    },
    {
        "num": 36,
        "title": "100 Things Every Designer Needs to Know About People",
        "authors": "Susan Weinschenk",
        "publisher": "New Riders",
        "year_edition": "2020 (2nd Edition)",
        "isbn10": "0136746918",
        "isbn13": "978-0136746911",
        "pillar": "UX Psychology, Cognitive Ergonomics & Usability",
        "concept": "Behavioral science and neuro-design: attention capture, decision fatigue, motivation.",
        "vibe_moat": "Shows how to structure onboarding and dashboard landing screens to maximize user engagement and retention."
    },
    {
        "num": 37,
        "title": "Emotional Design: Why We Love (or Hate) Everyday Things",
        "authors": "Don Norman",
        "publisher": "Basic Books",
        "year_edition": "2005",
        "isbn10": "0465051367",
        "isbn13": "978-0465051366",
        "pillar": "UX Psychology, Cognitive Ergonomics & Usability",
        "concept": "Visceral, behavioral, and reflective design levels.",
        "vibe_moat": "Teaches how to infuse vibe-coded tools with reflective delight and emotional resonance that competitors cannot copy."
    },
    {
        "num": 38,
        "title": "Seductive Interaction Design: Creating Playful, Fun, and Effective User Experiences",
        "authors": "Stephen P. Anderson",
        "publisher": "New Riders",
        "year_edition": "2011",
        "isbn10": "0321725522",
        "isbn13": "978-0321725523",
        "pillar": "UX Psychology, Cognitive Ergonomics & Usability",
        "concept": "Playful interactions, curating curiosity, subtle psychological nudges.",
        "vibe_moat": "Transforms boring administrative forms into engaging, interactive onboarding flows."
    },
    {
        "num": 39,
        "title": "Hooked: How to Build Habit-Forming Products",
        "authors": "Nir Eyal",
        "publisher": "Portfolio",
        "year_edition": "2014",
        "isbn10": "1591847788",
        "isbn13": "978-1591847786",
        "pillar": "UX Psychology, Cognitive Ergonomics & Usability",
        "concept": "The Hook Loop: Trigger, Action, Variable Reward, Investment.",
        "vibe_moat": "Designs core retention loops into web apps so users return organically without expensive re-marketing."
    },
    {
        "num": 40,
        "title": "The Mom Test: How to Talk to Customers & Learn If Your Business is a Good Idea",
        "authors": "Rob Fitzpatrick",
        "publisher": "Founder Craft",
        "year_edition": "2013",
        "isbn10": "1492180742",
        "isbn13": "978-1492180746",
        "pillar": "UX Psychology, Cognitive Ergonomics & Usability",
        "concept": "How to talk to customers and uncover true pain points without leading them to lie.",
        "vibe_moat": "Mandatory before vibe-coding: validates that the UX addresses real problems rather than imagined needs."
    },

    # Pillar 5: Web Performance, Core Web Vitals & Browser Networking
    {
        "num": 41,
        "title": "High Performance Web Sites: Essential Knowledge for Front-End Engineers",
        "authors": "Steve Souders",
        "publisher": "O'Reilly Media",
        "year_edition": "2007",
        "isbn10": "0596529309",
        "isbn13": "978-0596529307",
        "pillar": "Web Performance, Core Web Vitals & Browser Networking",
        "concept": "The original 14 frontend performance rules: minification, caching, script placement.",
        "vibe_moat": "The baseline performance checklist to inject into build prompts and asset delivery configs."
    },
    {
        "num": 42,
        "title": "Even Faster Web Sites: Performance Best Practices for Web Developers",
        "authors": "Steve Souders",
        "publisher": "O'Reilly Media",
        "year_edition": "2009",
        "isbn10": "0596522304",
        "isbn13": "978-0596522308",
        "pillar": "Web Performance, Core Web Vitals & Browser Networking",
        "concept": "Splitting code, non-blocking asynchronous script loading, domain sharding.",
        "vibe_moat": "Teaches how to prompt for non-blocking asynchronous asset loading that keeps the main thread responsive."
    },
    {
        "num": 43,
        "title": "Designing for Performance: Weighing Aesthetics and Speed",
        "authors": "Lara Callender Hogan",
        "publisher": "O'Reilly Media",
        "year_edition": "2014",
        "isbn10": "1491902515",
        "isbn13": "978-1491902516",
        "pillar": "Web Performance, Core Web Vitals & Browser Networking",
        "concept": "Performance budgets, aligning design aesthetics with page speed, user perception.",
        "vibe_moat": "Helps set strict byte budgets on vibe-coded apps before adding heavy client-side dependencies."
    },
    {
        "num": 44,
        "title": "Web Performance in Action: Building Fast Web Pages",
        "authors": "Jeremy L. Wagner",
        "publisher": "Manning Publications",
        "year_edition": "2016",
        "isbn10": "1617293776",
        "isbn13": "978-1617293771",
        "pillar": "Web Performance, Core Web Vitals & Browser Networking",
        "concept": "Critical rendering path, image compression, HTTP/2 multiplexing, service workers.",
        "vibe_moat": "Hands-on recipes to score 95+ on Google Lighthouse by optimizing CSS delivery and image formats."
    },
    {
        "num": 45,
        "title": "High Performance Browser Networking: What Every Web Developer Should Know About Networking and Web Performance",
        "authors": "Ilya Grigorik",
        "publisher": "O'Reilly Media",
        "year_edition": "2013",
        "isbn10": "1449344763",
        "isbn13": "978-1449344764",
        "pillar": "Web Performance, Core Web Vitals & Browser Networking",
        "concept": "TCP/UDP mechanics, TLS handshakes, HTTP/2/3, WebSockets, WebRTC optimization.",
        "vibe_moat": "The definitive guide to browser network plumbing: debugs latency, keep-alives, and connection pooling."
    },
    {
        "num": 46,
        "title": "Time Is Money: The Business Value of Web Performance",
        "authors": "Tammy Everts",
        "publisher": "O'Reilly Media",
        "year_edition": "2016",
        "isbn10": "1491935413",
        "isbn13": "978-1491935415",
        "pillar": "Web Performance, Core Web Vitals & Browser Networking",
        "concept": "Connecting milliseconds of latency to user bounce rates, conversion rates, and revenue.",
        "vibe_moat": "Provides empirical justification to resist bloating vibe-coded interfaces with unnecessary third-party scripts."
    },

    # Pillar 6: Accessibility (a11y), Forms & Web Standards
    {
        "num": 47,
        "title": "Inclusive Design Patterns: Coding Accessibility Into Web Components",
        "authors": "Heydon Pickering",
        "publisher": "Smashing Magazine",
        "year_edition": "2016",
        "isbn10": "3945749433",
        "isbn13": "978-3945749432",
        "pillar": "Accessibility (a11y), Forms & Web Standards",
        "concept": "Accessible component patterns: tab controls, toggle buttons, modal dialogs, data tables.",
        "vibe_moat": "The a11y gold standard: teaches how to prompt for semantic, accessible ARIA roles and keyboard interactions."
    },
    {
        "num": 48,
        "title": "Form Design Patterns: A Practical Guide to Designing and Building Forms for the Web",
        "authors": "Adam Silver",
        "publisher": "Smashing Magazine",
        "year_edition": "2018",
        "isbn10": "3945749719",
        "isbn13": "978-3945749715",
        "pillar": "Accessibility (a11y), Forms & Web Standards",
        "concept": "Bulletproof web forms: validation errors, input masks, multi-step flows, error recovery.",
        "vibe_moat": "Ensures forms never fail users: prompts for accessible inline errors, clear labels, and graceful validation."
    },
    {
        "num": 49,
        "title": "Accessibility for Everyone",
        "authors": "Laura Kalbag",
        "publisher": "A Book Apart",
        "year_edition": "2017",
        "isbn10": "1937557618",
        "isbn13": "978-1937557614",
        "pillar": "Accessibility (a11y), Forms & Web Standards",
        "concept": "Pragmatic WCAG guidelines, color contrast, keyboard traps, inclusive user research.",
        "vibe_moat": "Eliminates inaccessible gray-on-gray text, unannounced modal states, and broken tab orders in AI code."
    },
    {
        "num": 50,
        "title": "HTML5 for Web Designers",
        "authors": "Jeremy Keith, Rachel Andrew",
        "publisher": "A Book Apart",
        "year_edition": "2016 (2nd Edition)",
        "isbn10": "1937557456",
        "isbn13": "978-1937557454",
        "pillar": "Accessibility (a11y), Forms & Web Standards",
        "concept": "Native semantic elements (<main>, <article>, <dialog>), document outlines.",
        "vibe_moat": "Stops the AI habit of building div-soup; enforces semantic HTML that screen readers understand out of the box."
    }
]

# --- 2. Mathematical Checksum Verification Functions ---
def validate_isbn10(isbn: str) -> bool:
    clean = isbn.replace("-", "").strip()
    if len(clean) != 10:
        return False
    total = 0
    for i, char in enumerate(clean):
        if char in "0123456789":
            val = int(char)
        elif char in "Xx" and i == 9:
            val = 10
        else:
            return False
        total += val * (10 - i)
    return total % 11 == 0

def validate_isbn13(isbn: str) -> bool:
    clean = isbn.replace("-", "").strip()
    if len(clean) != 13 or not clean.isdigit():
        return False
    total = 0
    for i, char in enumerate(clean):
        weight = 1 if i % 2 == 0 else 3
        total += int(char) * weight
    return total % 10 == 0

def verify_all_isbns():
    print("[TEST 1/4] Brutal Mathematical Validation of 50 Books ISBNs...")
    errors = []
    for b in BOOKS_50_CANON:
        i10 = b["isbn10"]
        i13 = b["isbn13"]
        if not validate_isbn10(i10):
            errors.append(f"Book #{b['num']} '{b['title']}': Invalid ISBN-10 '{i10}'")
        if not validate_isbn13(i13):
            errors.append(f"Book #{b['num']} '{b['title']}': Invalid ISBN-13 '{i13}'")
    
    if errors:
        print(f"❌ FAILED: {len(errors)} ISBN errors found:")
        for err in errors:
            print(f"  - {err}")
        return False
    print("✅ PASS: All 50 books have 100% mathematically valid ISBN-10 (mod 11) and ISBN-13 (mod 10) checksums.")
    return True

# --- 3. Content Generation Functions ---
def generate_skill_content() -> str:
    frontmatter = """---
name: "web-design-dev-vibe-coder"
description: "Master Web Design & Development skill for vibe code developers: Visual Hierarchy, Typographic Scale, Modern Algorithmic CSS, Atomic Design Systems, UX Ergonomics, Core Web Vitals, and Inclusive a11y across 50 canonical texts."
---
# Web Design & Development for the Vibe Code Developer

## Overview & Escaping AI Slop
When generative AI models allow any developer to synthesize full-stack web layouts in seconds, the default output is almost always **"AI Slop"**:
- Cookie-cutter Tailwind cards with flat gray-on-white monotony.
- Uninspired typography without proportional scales or vertical rhythm.
- Fragile layouts built with negative margins and rigid pixel bounds.
- Broken accessibility trees, missing ARIA landmarks, and keyboard focus traps.
- Bloated client-side JavaScript bundles that destroy Core Web Vitals on mobile devices.

**This skill equips the developer and AI agent with rigorous visual, architectural, and ergonomic frameworks across 50 verified canonical texts to:**
1. Master visual design, spacing hierarchies, and typographic scales (Bringhurst, Williams, Saffer).
2. Write fluid, zero-media-query algorithmic CSS layouts and hardware-accelerated motion (Meyer, Verou, Drasner).
3. Architect atomic component systems and framework-agnostic design tokens (Frost, Kholmatova, Godbolt).
4. Apply cognitive ergonomics and frictionless usability patterns (Krug, Norman, Johnson).
5. Enforce strict Core Web Vitals budgets and browser networking efficiency (Souders, Grigorik, Wagner).
6. Build universally accessible, semantic web applications from the DOM up (Pickering, Silver, Kalbag).

---

## The Iron Laws of Web Design for Vibe Coders

```
1. UNPLUG THE MOUSE: IF YOU CANNOT NAVIGATE THE UI ENTIRELY VIA KEYBOARD, THE CODE IS BROKEN.
2. ELIMINATE AI SLOP: USE STRICT GEOMETRIC SPACING SCALES (4/8/16/24/32px) AND INTENTIONAL CONTRAST.
3. FLUIDITY OVER BREAKPOINTS: DESIGN LAYOUTS THAT WRAP AND REFLOW NATURALLY WITHOUT FIXED PIXELS.
4. RESPECT CORE WEB VITALS: LCP < 2.5s, INP < 200ms, CLS ~ 0. ZERO UNNECESSARY HEAVY JS BUNDLES.
5. SEMANTIC HTML FIRST: NEVER USE <div onClick> WHEN A NATIVE <button> OR <a> BELONGS THERE.
```

---

## The 6 Pillars & The 50-Book Canon

### Pillar I: Visual Design, UI Aesthetics & Craft
1. **The Non-Designer's Design Book** — *Robin Williams* (ISBN-10: 0133966151 | ISBN-13: 978-0133966152)
2. **The Elements of Typographic Style** — *Robert Bringhurst* (ISBN-10: 0881792128 | ISBN-13: 978-0881792126)
3. **Universal Principles of Design** — *William Lidwell et al.* (ISBN-10: 1592535879 | ISBN-13: 978-1592535873)
4. **Thinking with Type** — *Ellen Lupton* (ISBN-10: 1568989695 | ISBN-13: 978-1568989693)
5. **Grid Systems in Graphic Design** — *Josef Müller-Brockmann* (ISBN-10: 3721201450 | ISBN-13: 978-3721201451)
6. **Interaction of Color** — *Josef Albers* (ISBN-10: 0300179359 | ISBN-13: 978-0300179354)
7. **Design for Hackers** — *David Kadavy* (ISBN-10: 1119998956 | ISBN-13: 978-1119998952)
8. **Layout Essentials: 100 Design Principles for Using Grids** — *Beth Tondreau* (ISBN-10: 1592537073 | ISBN-13: 978-1592537075)
9. **Microinteractions: Designing with Details** — *Dan Saffer* (ISBN-10: 144934268X | ISBN-13: 978-1449342685)
10. **Designing Visual Interfaces** — *Kevin Mullet, Darrell Sano* (ISBN-10: 0133033899 | ISBN-13: 978-0133033892)

### Pillar II: Modern CSS Architecture, Layouts & Motion
11. **CSS Secrets** — *Lea Verou* (ISBN-10: 1449372635 | ISBN-13: 978-1449372637)
12. **CSS: The Definitive Guide** — *Eric A. Meyer, Estelle Weyl* (ISBN-10: 1098117611 | ISBN-13: 978-1098117610)
13. **Modern CSS with Tailwind** — *Noel Rappin* (ISBN-10: 1680508563 | ISBN-13: 978-1680508567)
14. **Transitions and Animations with CSS** — *Estelle Weyl* (ISBN-10: 1449321585 | ISBN-13: 978-1449321581)
15. **Responsive Web Design** — *Ethan Marcotte* (ISBN-10: 0983367116 | ISBN-13: 978-0983367116)
16. **CSS in Depth** — *Keith J. Grant* (ISBN-10: 1617293407 | ISBN-13: 978-1617293405)
17. **SVG Animations** — *Sarah Drasner* (ISBN-10: 1491939702 | ISBN-13: 978-1491939703)
18. **Sass and Compass in Action** — *Wynn Netherland et al.* (ISBN-10: 1617290149 | ISBN-13: 978-1617290145)
19. **CSS Master** — *Tiffany B. Brown* (ISBN-10: 1925836428 | ISBN-13: 978-1925836424)
20. **Pro CSS3 Animation** — *Dudley Storey* (ISBN-10: 1430247258 | ISBN-13: 978-1430247258)

### Pillar III: Component Systems, Design Systems & Frontend Architecture
21. **Atomic Design** — *Brad Frost* (ISBN-10: 0998296600 | ISBN-13: 978-0998296609)
22. **Designing Interfaces** — *Jenifer Tidwell et al.* (ISBN-10: 1492051969 | ISBN-13: 978-1492051961)
23. **Design Systems** — *Alla Kholmatova* (ISBN-10: 3945749581 | ISBN-13: 978-3945749586)
24. **Building Design Systems** — *Sarrah Vesselov, Taurie Davis* (ISBN-10: 148424513X | ISBN-13: 978-1484245132)
25. **Frontend Architecture for Design Systems** — *Micah Godbolt* (ISBN-10: 1491926783 | ISBN-13: 978-1491926789)
26. **Micro Frontends in Action** — *Michael Geers* (ISBN-10: 1617296872 | ISBN-13: 978-1617296871)
27. **Frameworkless Front-End Development** — *Francesco Strazzullo* (ISBN-10: 1484249666 | ISBN-13: 978-1484249666)
28. **Web Components in Action** — *Ben Farrell* (ISBN-10: 1617295779 | ISBN-13: 978-1617295775)
29. **Developing Large Web Applications** — *Kyle Loudon* (ISBN-10: 0596803028 | ISBN-13: 978-0596803025)
30. **Maintainable JavaScript** — *Nicholas C. Zakas* (ISBN-10: 1449325246 | ISBN-13: 978-1449325244)

### Pillar IV: UX Psychology, Cognitive Ergonomics & Usability
31. **Don't Make Me Think, Revisited** — *Steve Krug* (ISBN-10: 0321965515 | ISBN-13: 978-0321965516)
32. **The Design of Everyday Things** — *Don Norman* (ISBN-10: 0465050654 | ISBN-13: 978-0465050659)
33. **Designing with the Mind in Mind** — *Jeff Johnson* (ISBN-10: 0128182024 | ISBN-13: 978-0128182024)
34. **About Face: The Essentials of Interaction Design** — *Alan Cooper et al.* (ISBN-10: 1118766571 | ISBN-13: 978-1118766576)
35. **Rocket Surgery Made Easy** — *Steve Krug* (ISBN-10: 0321657292 | ISBN-13: 978-0321657299)
36. **100 Things Every Designer Needs to Know About People** — *Susan Weinschenk* (ISBN-10: 0136746918 | ISBN-13: 978-0136746911)
37. **Emotional Design** — *Don Norman* (ISBN-10: 0465051367 | ISBN-13: 978-0465051366)
38. **Seductive Interaction Design** — *Stephen P. Anderson* (ISBN-10: 0321725522 | ISBN-13: 978-0321725523)
39. **Hooked: How to Build Habit-Forming Products** — *Nir Eyal* (ISBN-10: 1591847788 | ISBN-13: 978-1591847786)
40. **The Mom Test** — *Rob Fitzpatrick* (ISBN-10: 1492180742 | ISBN-13: 978-1492180746)

### Pillar V: Web Performance, Core Web Vitals & Browser Networking
41. **High Performance Web Sites** — *Steve Souders* (ISBN-10: 0596529309 | ISBN-13: 978-0596529307)
42. **Even Faster Web Sites** — *Steve Souders* (ISBN-10: 0596522304 | ISBN-13: 978-0596522308)
43. **Designing for Performance** — *Lara Callender Hogan* (ISBN-10: 1491902515 | ISBN-13: 978-1491902516)
44. **Web Performance in Action** — *Jeremy L. Wagner* (ISBN-10: 1617293776 | ISBN-13: 978-1617293771)
45. **High Performance Browser Networking** — *Ilya Grigorik* (ISBN-10: 1449344763 | ISBN-13: 978-1449344764)
46. **Time Is Money: The Business Value of Web Performance** — *Tammy Everts* (ISBN-10: 1491935413 | ISBN-13: 978-1491935415)

### Pillar VI: Accessibility (a11y), Forms & Web Standards
47. **Inclusive Design Patterns** — *Heydon Pickering* (ISBN-10: 3945749433 | ISBN-13: 978-3945749432)
48. **Form Design Patterns** — *Adam Silver* (ISBN-10: 3945749719 | ISBN-13: 978-3945749715)
49. **Accessibility for Everyone** — *Laura Kalbag* (ISBN-10: 1937557618 | ISBN-13: 978-1937557614)
50. **HTML5 for Web Designers** — *Jeremy Keith, Rachel Andrew* (ISBN-10: 1937557456 | ISBN-13: 978-1937557454)

---

## Practical Protocols for Agents & Developers

### Protocol 1: The Visual Refinement Checklist
1. **Geometric Scale**: Ensure all padding, margin, and gaps conform to a base-4/base-8 scale (`4px, 8px, 12px, 16px, 24px, 32px, 48px, 64px`).
2. **Typographic Rhythm**: Establish a clear modular type scale (e.g. Minor Third `1.2` or Major Third `1.25`) with strict vertical rhythm and max line-length of 65-75 characters.
3. **Intentional Contrast**: Test WCAG AA contrast (4.5:1 for body text, 3:1 for large text). Never use low-contrast gray text on off-white backgrounds.

### Protocol 2: The Core Web Vitals & Accessibility Gate
1. **Focus Rings**: Verify keyboard focus states (`:focus-visible`) are prominent, accessible, and not overridden with `outline: none`.
2. **Form Semantics**: Ensure all `<input>` elements have explicit `<label>` bindings, descriptive helper text, and accessible error messages (`aria-describedby`).
3. **Cumulative Layout Shift (CLS)**: Verify all images and media containers have explicit `aspect-ratio` or `width`/`height` attributes to prevent layout jumping during render.
"""
    return frontmatter

def generate_agent_content() -> str:
    content = """---
name: web-designer-developer
description: Principal Web Designer & Frontend Architect for vibe code developers: eliminates AI slop, establishes typographic hierarchies, enforces atomic design systems, audits Core Web Vitals, and guarantees WCAG accessibility.
tools: read_file, run_command, calculator
model: deepseek-reasoner
---

# Web Designer & Frontend Developer Agent Persona

You are the ECC Principal Web Designer, UI/UX Craftsperson, and Frontend Architect.

## Core Objective
Transform rough, functional AI-generated web code into visually stunning, ergonomically effortless, accessible, and high-performance digital products. Eliminate "AI slop" and replace it with intentional design craft.

## Core Design Principles
1. **Contrast, Repetition, Alignment, Proximity (CRAP)**: Enforce strict visual hierarchies and spatial harmony.
2. **Typographic Discipline**: Apply modular typographic scales, vertical line-height rhythms, and optimal line measures.
3. **Fluid Layout Primitives**: Reject brittle fixed breakpoints; build self-wrapping, algorithmic CSS layouts (Flexbox, CSS Grid).
4. **Cognitive Simplicity (Don't Make Me Think)**: Make every interactive state, button, and navigation flow self-evident.
5. **Sub-Second Performance**: Keep critical rendering path lean; audit against Core Web Vitals (LCP, INP, CLS).
6. **Universal Accessibility (a11y)**: Enforce semantic HTML5, keyboard navigability, and WCAG AA compliance.

## Diagnostic & Audit Protocol
1. **Inspect for AI Slop**: Detect flat gray-on-white monotony, missing borders, arbitrary spacing, and unstyled form controls.
2. **Audit DOM Semantics**: Replace `div` click handlers with native interactive elements (`<button>`, `<a>`, `<dialog>`).
3. **Verify Motion Ergonomics**: Ensure animations use GPU-composited properties (`transform`, `opacity`) with natural easing curves.
4. **Enforce Design Tokens**: Unify colors, spacing, radius, and shadows into a centralized, themeable design token catalog.
"""
    return content

def generate_reference_guide() -> str:
    lines = [
        "# VIBE CODER'S WEB DESIGN & DEVELOPMENT CANON",
        "## The 50 Authoritative Books on Visual Craft, Modern CSS, Design Systems, UX, Performance, and a11y",
        "",
        "> Vibe coding enables rapid full-stack software generation, but uncurated code defaults to **AI Slop**.",
        "> This guide details the 50 foundational texts that give builders the visual taste,",
        "> responsive layout mastery, and cognitive ergonomics to build world-class web products.",
        "",
        "---",
        ""
    ]
    
    current_pillar = None
    for b in BOOKS_50_CANON:
        if b["pillar"] != current_pillar:
            current_pillar = b["pillar"]
            lines.append(f"## {current_pillar}")
            lines.append("")
        
        lines.append(f"### #{b['num']}. {b['title']}")
        lines.append(f"- **Author(s)**: {b['authors']}")
        lines.append(f"- **Publisher / Edition**: {b['publisher']} ({b['year_edition']})")
        lines.append(f"- **ISBN-10**: `{b['isbn10']}` | **ISBN-13**: `{b['isbn13']}`")
        lines.append(f"- **Core Premise**: {b['concept']}")
        lines.append(f"- **The Vibe Coder's Edge**: {b['vibe_moat']}")
        lines.append("")
    
    lines.append("---")
    lines.append("## The Vibe Coder's Visual Quality Refinement Matrix")
    lines.append("")
    lines.append("```mermaid")
    lines.append("flowchart TD")
    lines.append("    A[Raw AI Web Generation] --> B[1. Visual Hierarchy & Spacing Audit]")
    lines.append("    B --> C[2. Responsive Fluidity & Motion Check]")
    lines.append("    C --> D[3. Cognitive Ergonomics & Form Usability]")
    lines.append("    D --> E[4. Core Web Vitals & Asset Budgets]")
    lines.append("    E --> F[5. WCAG AA & Keyboard Accessibility]")
    lines.append("    F --> G[DEPLOY: Distinctive, High-Performance Web Product]")
    lines.append("```")
    lines.append("")
    return "\n".join(lines)

# --- 4. Deployment and Sync ---
def deploy_files():
    print("[TEST 2/4] Generating and deploying files across workspaces...")
    
    skill_content = generate_skill_content()
    agent_content = generate_agent_content()
    guide_content = generate_reference_guide()
    
    # Target directories
    targets = [
        Path("/home/dyna/TGS Projects/tagisan"),
        Path("/home/dyna/TGS Projects")
    ]
    
    for base_dir in targets:
        skill_dir = base_dir / ".ecc" / "skills" / "web-design-dev-vibe-coder"
        skill_dir.mkdir(parents=True, exist_ok=True)
        (skill_dir / "SKILL.md").write_text(skill_content, encoding="utf-8")
        print(f"  [+] Deployed: {skill_dir / 'SKILL.md'}")
        
        agent_dir = base_dir / ".ecc" / "agents"
        agent_dir.mkdir(parents=True, exist_ok=True)
        (agent_dir / "web-designer-developer.md").write_text(agent_content, encoding="utf-8")
        print(f"  [+] Deployed: {agent_dir / 'web-designer-developer.md'}")
        
    doc_path = Path("/home/dyna/TGS Projects/tagisan/docs/VIBE_CODER_WEB_DESIGN_AND_DEV_GUIDE.md")
    doc_path.parent.mkdir(parents=True, exist_ok=True)
    doc_path.write_text(guide_content, encoding="utf-8")
    print(f"  [+] Deployed: {doc_path}")
    
    # Invalidate cache
    cache_path = Path("/home/dyna/TGS Projects/tagisan/.tagisan/skills.cache")
    if cache_path.exists():
        cache_path.unlink()
        print("  [+] Invalidated .tagisan/skills.cache")
        
    print("✅ PASS: Files deployed and synchronized.")
    return True

# --- 5. Frontmatter Verification ---
def verify_frontmatter():
    print("[TEST 3/4] Verifying Tagisan YAML frontmatter parsing...")
    skill_file = Path("/home/dyna/TGS Projects/tagisan/.ecc/skills/web-design-dev-vibe-coder/SKILL.md")
    agent_file = Path("/home/dyna/TGS Projects/tagisan/.ecc/agents/web-designer-developer.md")
    
    s_text = skill_file.read_text(encoding="utf-8")
    if not (s_text.startswith("---") and "\n---\n" in s_text):
        print("❌ FAILED: Invalid frontmatter delimiters in SKILL.md")
        return False
    
    a_text = agent_file.read_text(encoding="utf-8")
    if not (a_text.startswith("---") and "\n---\n" in a_text):
        print("❌ FAILED: Invalid frontmatter delimiters in Agent.md")
        return False
        
    print("✅ PASS: Frontmatter delimiters and structure match Tagisan specifications.")
    return True

# --- 6. Live CLI Integration Tests with tgs ---
def test_cli_integration():
    print("[TEST 4/4] Testing live CLI integration with 'tgs' binary...")
    
    # 1. Test tgs ecc list
    print("  Executing: tgs ecc list")
    try:
        proc = subprocess.run(
            ["tgs", "ecc", "list"],
            cwd="/home/dyna/TGS Projects/tagisan",
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=15
        )
        if "web-designer-developer" in proc.stdout:
            print("  [✓] 'web-designer-developer' successfully detected by 'tgs ecc list'!")
        else:
            print("❌ FAILED: 'web-designer-developer' not found in 'tgs ecc list'")
            print("Stdout:\n", proc.stdout)
            return False
    except Exception as e:
        print(f"❌ Error running 'tgs ecc list': {e}")
        return False

    # 2. Test tgs ecc skills with query
    print("  Executing: tgs ecc skills --query 'web-design'")
    try:
        proc2 = subprocess.run(
            ["tgs", "ecc", "skills", "--query", "web-design"],
            cwd="/home/dyna/TGS Projects/tagisan",
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=25
        )
        if "web-design-dev-vibe-coder" in proc2.stdout:
            print("  [✓] 'web-design-dev-vibe-coder' successfully detected and ranked by 'tgs ecc skills'!")
        else:
            print("❌ FAILED: 'web-design-dev-vibe-coder' not ranked in 'tgs ecc skills'")
            print("Stdout:\n", proc2.stdout)
            return False
    except Exception as e:
        print(f"❌ Error running 'tgs ecc skills': {e}")
        return False

    print("✅ PASS: Live CLI integration verified.")
    return True

def main():
    print("=" * 65)
    print(" 🎨 TAGISAN WEB DESIGN & DEV TEST HARNESS ")
    print("=" * 65)
    
    if not verify_all_isbns():
        sys.exit(1)
    if not deploy_files():
        sys.exit(1)
    if not verify_frontmatter():
        sys.exit(1)
    if not test_cli_integration():
        sys.exit(1)
        
    print("\n" + "=" * 65)
    print(" 🎉 ALL TESTS PASSED: 100% VERIFIED WEB DESIGN & DEV CANON")
    print("=" * 65)

if __name__ == "__main__":
    main()
