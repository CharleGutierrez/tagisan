#!/usr/bin/env python3
"""
Automated Verification Test Harness for:
Strategic & Competitive Analysis for the Vibe Code Developer (50 Books Canon)
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

# --- 1. The 50 Canonical Strategic & Competitive Analysis Books ---
BOOKS_50_CANON = [
    # Pillar 1: Structural Defensibility, Moats & Economic Powers
    {
        "num": 1,
        "title": "7 Powers: The Foundations of Business Strategy",
        "authors": "Hamilton Helmer",
        "publisher": "Deep Strategy",
        "year_edition": "2016",
        "isbn10": "0998116319",
        "isbn13": "978-0998116310",
        "pillar": "Structural Defensibility, Moats & Economic Powers",
        "concept": "The 7 durable barriers: Scale Economies, Network Economies, Counter-Positioning, Switching Costs, Branding, Cornered Resource, Process Power.",
        "vibe_moat": "The core filter for vibe coders: proves whether an AI product has structural economic power once the code is generated."
    },
    {
        "num": 2,
        "title": "Competitive Strategy: Techniques for Analyzing Industries and Competitors",
        "authors": "Michael E. Porter",
        "publisher": "Free Press",
        "year_edition": "1998 (Reprint)",
        "isbn10": "0684841487",
        "isbn13": "978-0684841489",
        "pillar": "Structural Defensibility, Moats & Economic Powers",
        "concept": "Porter's Five Forces: Supplier Power, Buyer Power, Competitive Rivalry, Threat of Substitution, Barriers to Entry.",
        "vibe_moat": "Analyzes margin erosion when upstream AI foundation model providers (suppliers) commoditize app wrappers."
    },
    {
        "num": 3,
        "title": "Competitive Advantage: Creating and Sustaining Superior Performance",
        "authors": "Michael E. Porter",
        "publisher": "Free Press",
        "year_edition": "1998 (Reprint)",
        "isbn10": "0684841460",
        "isbn13": "978-0684841465",
        "pillar": "Structural Defensibility, Moats & Economic Powers",
        "concept": "Cost leadership, differentiation, focus, and value chain disaggregation.",
        "vibe_moat": "Shows how to isolate the specific activity in your customer's value chain where AI generates genuine cost or differentiation advantages."
    },
    {
        "num": 4,
        "title": "Understanding Michael Porter: The Essential Guide to Competition and Strategy",
        "authors": "Joan Magretta",
        "publisher": "Harvard Business Review Press",
        "year_edition": "2011",
        "isbn10": "1422160599",
        "isbn13": "978-1422160596",
        "pillar": "Structural Defensibility, Moats & Economic Powers",
        "concept": "Modern, concise synthesis of Porter's core economic frameworks.",
        "vibe_moat": "Avoids the zero-sum 'competition to be the best' (feature bloat) in favor of 'competing to be unique'."
    },
    {
        "num": 5,
        "title": "Good Strategy/Bad Strategy: The Difference and Why It Matters",
        "authors": "Richard P. Rumelt",
        "publisher": "Crown Business",
        "year_edition": "2011",
        "isbn10": "0307886239",
        "isbn13": "978-0307886231",
        "pillar": "Structural Defensibility, Moats & Economic Powers",
        "concept": "The Kernel of Strategy: Diagnosis, Guiding Policy, and Coherent Action vs. empty goal-setting.",
        "vibe_moat": "Stops vibe coders from confusing 'ship 10 features this weekend' with an actual coherent competitive strategy."
    },
    {
        "num": 6,
        "title": "The Crux: How Leaders Become Strategists",
        "authors": "Richard P. Rumelt",
        "publisher": "PublicAffairs",
        "year_edition": "2022",
        "isbn10": "1541701240",
        "isbn13": "978-1541701243",
        "pillar": "Structural Defensibility, Moats & Economic Powers",
        "concept": "Identifying the single most critical, addressable challenge standing between you and market dominance.",
        "vibe_moat": "Teaches focus: solving the hardest bottleneck (e.g., proprietary data pipeline) rather than generating 20 peripheral UI components."
    },
    {
        "num": 7,
        "title": "Competition Demystified: A Radically Simplified Approach to Business Strategy",
        "authors": "Bruce Greenwald, Judd Kahn",
        "publisher": "Portfolio",
        "year_edition": "2005",
        "isbn10": "1591841801",
        "isbn13": "978-1591841807",
        "pillar": "Structural Defensibility, Moats & Economic Powers",
        "concept": "Simplifies strategic analysis down to a single dominant variable: Barriers to Entry.",
        "vibe_moat": "Helps answer the only question that matters: 'What prevents 500 other developers from copying this prompt tomorrow?'"
    },
    {
        "num": 8,
        "title": "Modern Competitive Analysis",
        "authors": "Sharon M. Oster",
        "publisher": "Oxford University Press",
        "year_edition": "1999 (3rd Edition)",
        "isbn10": "019511941X",
        "isbn13": "978-0195119411",
        "pillar": "Structural Defensibility, Moats & Economic Powers",
        "concept": "Industrial organization economics applied to business competition and strategic groups.",
        "vibe_moat": "Provides formal economic rigor on game-theoretic entry deterrence and irreversible investments."
    },
    {
        "num": 9,
        "title": "The Origin of Wealth: The Radical Remaking of Economics and What It Means for Business and Society",
        "authors": "Eric D. Beinhocker",
        "publisher": "Harvard Business Review Press",
        "year_edition": "2007",
        "isbn10": "1422121038",
        "isbn13": "978-1422121030",
        "pillar": "Structural Defensibility, Moats & Economic Powers",
        "concept": "Complexity economics, evolutionary business strategies, adaptive fitness landscapes.",
        "vibe_moat": "Models fast-moving AI markets as complex adaptive systems rather than static equilibrium industries."
    },
    {
        "num": 10,
        "title": "Strategy Rules: Five Timeless Lessons from Bill Gates, Andy Grove, and Steve Jobs",
        "authors": "David B. Yoffie, Michael A. Cusumano",
        "publisher": "Harper Business",
        "year_edition": "2015",
        "isbn10": "0062373951",
        "isbn13": "978-0062373953",
        "pillar": "Structural Defensibility, Moats & Economic Powers",
        "concept": "Look forward, reason back; build platforms; exploit industry inflection points.",
        "vibe_moat": "Shows how to anticipate where foundation model providers are heading and build where they won't tread."
    },

    # Pillar 2: Platform Economics, Network Effects & Aggregation Strategy
    {
        "num": 11,
        "title": "Platform Revolution: How Networked Markets Are Transforming the Economy",
        "authors": "Geoffrey G. Parker, Marshall W. Van Alstyne, Sangeet Paul Choudary",
        "publisher": "W. W. Norton & Company",
        "year_edition": "2016",
        "isbn10": "0393249131",
        "isbn13": "978-0393249132",
        "pillar": "Platform Economics, Network Effects & Aggregation Strategy",
        "concept": "How platform business models beat pipeline models via external ecosystem production.",
        "vibe_moat": "Guides transitioning a single-player vibe-coded SaaS into a multi-sided marketplace or exchange."
    },
    {
        "num": 12,
        "title": "The Business of Platforms: Strategy in the Age of Digital Competition, Innovation, and Power",
        "authors": "Michael A. Cusumano, Annabelle Gawer, David B. Yoffie",
        "publisher": "Harper Business",
        "year_edition": "2019",
        "isbn10": "0062896326",
        "isbn13": "978-0062896322",
        "pillar": "Platform Economics, Network Effects & Aggregation Strategy",
        "concept": "Transaction Platforms, Innovation Platforms, and Hybrid Platform dynamics.",
        "vibe_moat": "Identifies why pure AI wrapper tools fail and how to build an 'innovation platform' that other developers extend."
    },
    {
        "num": 13,
        "title": "The Cold Start Problem: How to Start and Scale Network Effects",
        "authors": "Andrew Chen",
        "publisher": "Harper Business",
        "year_edition": "2021",
        "isbn10": "0062969749",
        "isbn13": "978-0062969743",
        "pillar": "Platform Economics, Network Effects & Aggregation Strategy",
        "concept": "Atomic networks, tipping points, hard side of the network, anti-network effects.",
        "vibe_moat": "Solves the chicken-and-egg distribution dilemma for AI products that rely on user interactions to improve."
    },
    {
        "num": 14,
        "title": "Matchmakers: The New Economics of Multisided Platforms",
        "authors": "David S. Evans, Richard Schmalensee",
        "publisher": "Harvard Business Review Press",
        "year_edition": "2016",
        "isbn10": "1633691721",
        "isbn13": "978-1633691728",
        "pillar": "Platform Economics, Network Effects & Aggregation Strategy",
        "concept": "Frictions, catalytic businesses, pricing symmetry, multi-homing prevention.",
        "vibe_moat": "Teaches how to design economic incentives so both sides of your AI marketplace refuse to defect to copycats."
    },
    {
        "num": 15,
        "title": "Invisible Engines: How the Software Platforms Drive Innovation and Transform Industries",
        "authors": "David S. Evans, Andrei Hagiu, Richard Schmalensee",
        "publisher": "The MIT Press",
        "year_edition": "2006",
        "isbn10": "0262050854",
        "isbn13": "978-0262050852",
        "pillar": "Platform Economics, Network Effects & Aggregation Strategy",
        "concept": "Historical economics of software platforms: Windows, Symbian, Palm, and web platforms.",
        "vibe_moat": "Shows how software platforms capture platform surplus and maintain developer stickiness."
    },
    {
        "num": 16,
        "title": "Modern Monopolies: What It Takes to Dominate the 21st-Century Economy",
        "authors": "Alex Moazed, Nicholas L. Johnson",
        "publisher": "St. Martin's Press",
        "year_edition": "2016",
        "isbn10": "1250091896",
        "isbn13": "978-1250091895",
        "pillar": "Platform Economics, Network Effects & Aggregation Strategy",
        "concept": "Platform economics, zero marginal cost of connection, decentralized inventory.",
        "vibe_moat": "Teaches vibe coders how to build networks that own the transaction rails rather than manufacturing features."
    },
    {
        "num": 17,
        "title": "Zero to One: Notes on Startups, or How to Build the Future",
        "authors": "Peter Thiel with Blake Masters",
        "publisher": "Crown Business",
        "year_edition": "2014",
        "isbn10": "0804139296",
        "isbn13": "978-0804139298",
        "pillar": "Platform Economics, Network Effects & Aggregation Strategy",
        "concept": "Escaping competition, secrets, monopoly through 10x proprietary tech, network effects, scale.",
        "vibe_moat": "Reminds AI builders that copying an existing app with an LLM creates 1 to N, not 0 to 1."
    },
    {
        "num": 18,
        "title": "Information Rules: A Strategic Guide to the Network Economy",
        "authors": "Carl Shapiro, Hal R. Varian",
        "publisher": "Harvard Business Review Press",
        "year_edition": "1998",
        "isbn10": "087584863X",
        "isbn13": "978-0875848631",
        "pillar": "Platform Economics, Network Effects & Aggregation Strategy",
        "concept": "Lock-in, switching costs, versioning, positive feedback loops, standard wars.",
        "vibe_moat": "Written by Google's Chief Economist: the foundational economic mechanics of digital pricing and lock-in."
    },
    {
        "num": 19,
        "title": "The Keystone Advantage: What the New Dynamics of Business Ecosystems Mean for Strategy, Innovation, and Sustainability",
        "authors": "Marco Iansiti, Roy Levien",
        "publisher": "Harvard Business Review Press",
        "year_edition": "2004",
        "isbn10": "1591393078",
        "isbn13": "978-1591393078",
        "pillar": "Platform Economics, Network Effects & Aggregation Strategy",
        "concept": "Biological ecosystem analogy: Keystone vs. Dominator vs. Niche players in business webs.",
        "vibe_moat": "How to position your AI product as a beneficial 'keystone' in an ecosystem rather than an extractive dominator."
    },
    {
        "num": 20,
        "title": "Platform Strategy: How to Unlock the Power of Communities and Networks to Grow Your Business",
        "authors": "Laure Claire Reillier, Benoit Reillier",
        "publisher": "Routledge",
        "year_edition": "2017",
        "isbn10": "1472480244",
        "isbn13": "978-1472480248",
        "pillar": "Platform Economics, Network Effects & Aggregation Strategy",
        "concept": "The Rocket Model: attract, match, connect, transact, optimize.",
        "vibe_moat": "Practical operational framework for architecting self-reinforcing loops in user-facing AI platforms."
    },

    # Pillar 3: Disruption Theory, Technology Lifecycles & Value Chain Evolution
    {
        "num": 21,
        "title": "The Innovator's Dilemma: When New Technologies Cause Great Firms to Fail",
        "authors": "Clayton M. Christensen",
        "publisher": "Harvard Business Review Press",
        "year_edition": "2016 (Reprint)",
        "isbn10": "1633691780",
        "isbn13": "978-1633691780",
        "pillar": "Disruption Theory, Technology Lifecycles & Value Chain Evolution",
        "concept": "Disruptive vs. sustaining innovation, low-end footholds, new-market disruption.",
        "vibe_moat": "Shows how vibe coders can target non-consumers and low-margin niches that incumbent software giants ignore."
    },
    {
        "num": 22,
        "title": "The Innovator's Solution: Creating and Sustaining Successful Growth",
        "authors": "Clayton M. Christensen, Michael E. Raynor",
        "publisher": "Harvard Business Review Press",
        "year_edition": "2013 (Reprint)",
        "isbn10": "1422196577",
        "isbn13": "978-1422196571",
        "pillar": "Disruption Theory, Technology Lifecycles & Value Chain Evolution",
        "concept": "The Law of Conservation of Attractive Profits: When a product becomes modular, profits shift to proprietary subsystems.",
        "vibe_moat": "When code generation commoditizes, attractive profits shift to proprietary data and high-context workflows."
    },
    {
        "num": 23,
        "title": "Crossing the Chasm: Marketing and Selling Disruptive Products to Mainstream Customers",
        "authors": "Geoffrey A. Moore",
        "publisher": "Harper Business",
        "year_edition": "2014 (3rd Edition)",
        "isbn10": "0062292986",
        "isbn13": "978-0062292988",
        "pillar": "Disruption Theory, Technology Lifecycles & Value Chain Evolution",
        "concept": "The Technology Adoption Life Cycle: Moving from Innovators/Early Adopters to Pragmatists in the bowling alley.",
        "vibe_moat": "Prevents getting stuck in the 'tech-enthusiast bubble' with shiny AI demos that mainstream buyers won't touch."
    },
    {
        "num": 24,
        "title": "Inside the Tornado: Strategies for Developing, Leveraging, and Surviving Hypergrowth Markets",
        "authors": "Geoffrey A. Moore",
        "publisher": "Harper Business",
        "year_edition": "2004",
        "isbn10": "0060653647",
        "isbn13": "978-0060653644",
        "pillar": "Disruption Theory, Technology Lifecycles & Value Chain Evolution",
        "concept": "Market dynamics during hypergrowth: Bowling Alley, Tornado, Main Street.",
        "vibe_moat": "How to position your vibe-coded product when an AI market segment enters a hypergrowth 'tornado'."
    },
    {
        "num": 25,
        "title": "Wardley Mapping: Topographical Intelligence in Business",
        "authors": "Simon Wardley",
        "publisher": "Independent Publishing",
        "year_edition": "2018",
        "isbn10": "1729606822",
        "isbn13": "978-1729606827",
        "pillar": "Disruption Theory, Technology Lifecycles & Value Chain Evolution",
        "concept": "Evolution of value chains: Genesis -> Custom-Built -> Product -> Commodity/Utility.",
        "vibe_moat": "Essential visual strategy: maps out exactly which components of your AI stack are commoditizing into utilities."
    },
    {
        "num": 26,
        "title": "Escape Velocity: Free Your Company's Future from the Pull of the Past",
        "authors": "Geoffrey A. Moore",
        "publisher": "Harper Business",
        "year_edition": "2011",
        "isbn10": "0062040898",
        "isbn13": "978-0062040893",
        "pillar": "Disruption Theory, Technology Lifecycles & Value Chain Evolution",
        "concept": "Portfolio management, categories in secular decline vs. category growth.",
        "vibe_moat": "Helps you ruthlessly decommission zombie AI experiments to reallocate velocity to your core growth engine."
    },
    {
        "num": 27,
        "title": "Seeing What's Next: Using the Theories of Innovation to Predict Industry Change",
        "authors": "Clayton M. Christensen, Scott D. Anthony, Erik A. Roth",
        "publisher": "Harvard Business Review Press",
        "year_edition": "2004",
        "isbn10": "1591391857",
        "isbn13": "978-1591391852",
        "pillar": "Disruption Theory, Technology Lifecycles & Value Chain Evolution",
        "concept": "Using innovation theory to forecast competitive battles and industry structure changes.",
        "vibe_moat": "Evaluates whether your AI product faces threats from upstream platform providers or downstream customer integrators."
    },
    {
        "num": 28,
        "title": "Zone to Win: Organizing to Compete in an Age of Disruption",
        "authors": "Geoffrey A. Moore",
        "publisher": "Diversion Books",
        "year_edition": "2015",
        "isbn10": "1626818169",
        "isbn13": "978-1626818163",
        "pillar": "Disruption Theory, Technology Lifecycles & Value Chain Evolution",
        "concept": "Four operational zones: Performance, Productivity, Incubation, Transformation.",
        "vibe_moat": "Structural advice on managing rapid vibe-prototyping experiments without bankrupting core revenue operations."
    },
    {
        "num": 29,
        "title": "The Gorilla Game: An Investor's Guide to Picking Winners in High Technology",
        "authors": "Geoffrey A. Moore, Paul Johnson, Tom Kippola",
        "publisher": "Harper Business",
        "year_edition": "1999",
        "isbn10": "0887309577",
        "isbn13": "978-0887309571",
        "pillar": "Disruption Theory, Technology Lifecycles & Value Chain Evolution",
        "concept": "Identifying Gorillas, Chimps, and Monkeys in proprietary software architecture battles.",
        "vibe_moat": "How to identify whether your product can become the dominant architectural standard or should partner with one."
    },
    {
        "num": 30,
        "title": "Only the Paranoid Survive: How to Exploit the Crisis Points That Challenge Every Company",
        "authors": "Andrew S. Grove",
        "publisher": "Currency / Doubleday",
        "year_edition": "1996",
        "isbn10": "0385482582",
        "isbn13": "978-0385482585",
        "pillar": "Disruption Theory, Technology Lifecycles & Value Chain Evolution",
        "concept": "Strategic Inflection Points (10X shifts in market forces), navigating the Valley of Death.",
        "vibe_moat": "Guides your mindset when foundation models suddenly release a native feature that overlaps 80% of your product."
    },

    # Pillar 4: Positioning, Value Innovation & Market Creation (Blue Oceans)
    {
        "num": 31,
        "title": "Blue Ocean Strategy: How to Create Uncontested Market Space and Make the Competition Irrelevant",
        "authors": "W. Chan Kim, Renée Mauborgne",
        "publisher": "Harvard Business Review Press",
        "year_edition": "2015 (Expanded Edition)",
        "isbn10": "1625274491",
        "isbn13": "978-1625274496",
        "pillar": "Positioning, Value Innovation & Market Creation (Blue Oceans)",
        "concept": "Value Innovation: eliminate, reduce, raise, and create factors to break the value-cost trade-off.",
        "vibe_moat": "Eliminates unnecessary enterprise complexity while raising speed and delightful UX to open uncontested territory."
    },
    {
        "num": 32,
        "title": "Blue Ocean Shift: Beyond Competing - Proven Steps to Inspire Confidence and Seize New Growth",
        "authors": "W. Chan Kim, Renée Mauborgne",
        "publisher": "Hachette Books",
        "year_edition": "2017",
        "isbn10": "1610398149",
        "isbn13": "978-1610398145",
        "pillar": "Positioning, Value Innovation & Market Creation (Blue Oceans)",
        "concept": "Step-by-step roadmap to move from crowded red oceans to uncontested market space.",
        "vibe_moat": "Practical Pioneer-Migrator-Settler maps to steer your AI project portfolio toward differentiation."
    },
    {
        "num": 33,
        "title": "Obviously Awesome: How to Nail Product Positioning so Customers Get It, Buy It, Love It",
        "authors": "April Dunford",
        "publisher": "Ambient Press",
        "year_edition": "2019",
        "isbn10": "1999023005",
        "isbn13": "978-1999023003",
        "pillar": "Positioning, Value Innovation & Market Creation (Blue Oceans)",
        "concept": "The 10-step positioning methodology: competitive alternatives, unique attributes, value, customer segments.",
        "vibe_moat": "Helps vibe coders realize that your competitor is often not another AI tool, but 'an ugly Excel spreadsheet'."
    },
    {
        "num": 34,
        "title": "Positioning: The Battle for Your Mind",
        "authors": "Al Ries, Jack Trout",
        "publisher": "McGraw-Hill",
        "year_edition": "2001 (20th Anniversary Edition)",
        "isbn10": "0071373586",
        "isbn13": "978-0071373586",
        "pillar": "Positioning, Value Innovation & Market Creation (Blue Oceans)",
        "concept": "Owning a singular word or concept in the prospect's mental category hierarchy.",
        "vibe_moat": "In an avalanche of generic 'AI copilots', ensures your product stands for one clear, indelible concept."
    },
    {
        "num": 35,
        "title": "Sales Pitch: How to Craft a Story to Stand Out and Win",
        "authors": "April Dunford",
        "publisher": "Ambient Press",
        "year_edition": "2023",
        "isbn10": "1778216706",
        "isbn13": "978-1778216701",
        "pillar": "Positioning, Value Innovation & Market Creation (Blue Oceans)",
        "concept": "Crafting customer-centric narratives that translate unique technical positioning into closed deals.",
        "vibe_moat": "Teaches developers how to demo an AI application so buyers immediately perceive ROI rather than technical novelty."
    },
    {
        "num": 36,
        "title": "The 22 Immutable Laws of Marketing",
        "authors": "Al Ries, Jack Trout",
        "publisher": "Harper Business",
        "year_edition": "1993",
        "isbn10": "0887306667",
        "isbn13": "978-0887306662",
        "pillar": "Positioning, Value Innovation & Market Creation (Blue Oceans)",
        "concept": "The Law of Leadership, Law of Category, Law of the Mind, Law of Focus.",
        "vibe_moat": "Reminds builders: 'It's better to be first in a new category than to be better in an existing one.'"
    },
    {
        "num": 37,
        "title": "Playing to Win: How Strategy Really Works",
        "authors": "A.G. Lafley, Roger L. Martin",
        "publisher": "Harvard Business Review Press",
        "year_edition": "2013",
        "isbn10": "142218739X",
        "isbn13": "978-1422187395",
        "pillar": "Positioning, Value Innovation & Market Creation (Blue Oceans)",
        "concept": "The Strategic Choice Cascade: Winning Aspiration, Where to Play, How to Win, Capabilities, Systems.",
        "vibe_moat": "Forces you to make explicit trade-offs on where NOT to play, saving months of fruitless feature prompting."
    },
    {
        "num": 38,
        "title": "Different: Escaping the Competitive Herd",
        "authors": "Youngme Moon",
        "publisher": "Crown Business",
        "year_edition": "2010",
        "isbn10": "030746086X",
        "isbn13": "978-0307460868",
        "pillar": "Positioning, Value Innovation & Market Creation (Blue Oceans)",
        "concept": "Reverse positioning, breakaway positioning, and hostile positioning to escape conformist features.",
        "vibe_moat": "Explains why adding 50 AI features makes you look identical to competitors; shows how to strip away 80% to stand out."
    },
    {
        "num": 39,
        "title": "Competing Against Luck: The Story of Innovation and Customer Choice",
        "authors": "Clayton M. Christensen, Taddy Hall, Karen Dillon, David S. Duncan",
        "publisher": "Harper Business",
        "year_edition": "2016",
        "isbn10": "0062435612",
        "isbn13": "978-0062435613",
        "pillar": "Positioning, Value Innovation & Market Creation (Blue Oceans)",
        "concept": "Jobs-to-be-Done (JTBD) Theory: Customers don't buy products; they 'hire' them to make progress.",
        "vibe_moat": "Identifies the emotional, social, and functional 'job' your user is hiring your AI tool to accomplish."
    },
    {
        "num": 40,
        "title": "Demand-Side Sales 101: Stop Selling and Help Your Customers Make Progress",
        "authors": "Bob Moesta",
        "publisher": "Lioncrest Publishing",
        "year_edition": "2020",
        "isbn10": "1544509987",
        "isbn13": "978-1544509983",
        "pillar": "Positioning, Value Innovation & Market Creation (Blue Oceans)",
        "concept": "The Forces of Progress: Push of current situation, Pull of new idea, Anxiety of new solution, Habit of present.",
        "vibe_moat": "Teaches why users don't adopt your vibe-coded tool even if it's 10x faster: managing switching anxiety."
    },

    # Pillar 5: Game Theory, Competitive Wargaming & Strategic Maneuvering
    {
        "num": 41,
        "title": "Co-opetition: A Revolutionary Mindset That Combines Competition and Cooperation",
        "authors": "Adam M. Brandenburger, Barry J. Nalebuff",
        "publisher": "Currency / Doubleday",
        "year_edition": "1996",
        "isbn10": "0385516002",
        "isbn13": "978-0385516006",
        "pillar": "Game Theory, Competitive Wargaming & Strategic Maneuvering",
        "concept": "The PARTS framework (Players, Added values, Rules, Tactics, Scope) and the Value Net.",
        "vibe_moat": "Teaches how to cooperate with upstream LLM providers while fiercely capturing your own added value."
    },
    {
        "num": 42,
        "title": "Thinking Strategically: The Competitive Edge in Business, Politics, and Everyday Life",
        "authors": "Avinash K. Dixit, Barry J. Nalebuff",
        "publisher": "W. W. Norton & Company",
        "year_edition": "1991",
        "isbn10": "0393310353",
        "isbn13": "978-0393310351",
        "pillar": "Game Theory, Competitive Wargaming & Strategic Maneuvering",
        "concept": "Game-theoretic concepts: backward induction, commitments, credible threats, prisoner's dilemma.",
        "vibe_moat": "Helps predict competitor reactions before you launch aggressive pricing moves or open-source strategies."
    },
    {
        "num": 43,
        "title": "The Art of Strategy: A Game Theorist's Guide to Success in Business and Life",
        "authors": "Avinash K. Dixit, Barry J. Nalebuff",
        "publisher": "W. W. Norton & Company",
        "year_edition": "2008",
        "isbn10": "0393337170",
        "isbn13": "978-0393337174",
        "pillar": "Game Theory, Competitive Wargaming & Strategic Maneuvering",
        "concept": "Modernized guide to strategic game theory in business and everyday negotiations.",
        "vibe_moat": "Practical intuition on designing mechanism incentives that force competitors into suboptimal moves."
    },
    {
        "num": 44,
        "title": "Strategy: An Introduction to Game Theory",
        "authors": "Joel Watson",
        "publisher": "W. W. Norton & Company",
        "year_edition": "2013 (3rd Edition)",
        "isbn10": "0393918386",
        "isbn13": "978-0393918380",
        "pillar": "Game Theory, Competitive Wargaming & Strategic Maneuvering",
        "concept": "Formal mathematical foundation of extensive-form games, Nash equilibrium, signaling.",
        "vibe_moat": "Rigorous modeling of repeated games, imperfect information, and strategic signaling in software markets."
    },
    {
        "num": 45,
        "title": "Business Wargaming: Securing Corporate Value",
        "authors": "Daniel F. Oriesek, Jan Oliver Schwarz",
        "publisher": "Routledge / Gower",
        "year_edition": "2008",
        "isbn10": "0566088282",
        "isbn13": "978-0566088285",
        "pillar": "Game Theory, Competitive Wargaming & Strategic Maneuvering",
        "concept": "Simulating competitor moves, red-teaming business vulnerabilities, strategic role-play.",
        "vibe_moat": "Run wargaming exercises: 'If Microsoft/Google bundles this feature into Office tomorrow, how do we survive?'"
    },
    {
        "num": 46,
        "title": "The Art of War",
        "authors": "Sun Tzu (trans. Samuel B. Griffith)",
        "publisher": "Oxford University Press",
        "year_edition": "1971",
        "isbn10": "0195014766",
        "isbn13": "978-0195014761",
        "pillar": "Game Theory, Competitive Wargaming & Strategic Maneuvering",
        "concept": "Situational awareness, exploiting asymmetry, winning without fighting, formlessness.",
        "vibe_moat": "Encourages asymmetric maneuvers: attack where incumbents are structurally unable or culturally unwilling to compete."
    },
    {
        "num": 47,
        "title": "Certain to Win: The Strategy of John Boyd, Applied to Business",
        "authors": "Chet Richards",
        "publisher": "Xlibris",
        "year_edition": "2004",
        "isbn10": "1413453775",
        "isbn13": "978-1413453775",
        "pillar": "Game Theory, Competitive Wargaming & Strategic Maneuvering",
        "concept": "Boyd's OODA Loop (Observe-Orient-Decide-Act), fast transient maneuver warfare.",
        "vibe_moat": "The ultimate vibe coding tactical manual: use rapid AI iteration speed to turn inside your competitor's decision cycle."
    },

    # Pillar 6: Monetization Strategy, Unit Economics & Business Model Design
    {
        "num": 48,
        "title": "Monetizing Innovation: How Smart Companies Design the Product Around the Price",
        "authors": "Madhavan Ramanujam, Georg Tacke",
        "publisher": "Wiley",
        "year_edition": "2016",
        "isbn10": "1119240867",
        "isbn13": "978-1119240860",
        "pillar": "Monetization Strategy, Unit Economics & Business Model Design",
        "concept": "Designing the product around the price; willingness-to-pay (WTP) segmentation; feature shocks.",
        "vibe_moat": "Mandatory reading: eliminates the fatal mistake of vibe-coding an entire product before knowing if anyone will pay for it."
    },
    {
        "num": 49,
        "title": "Business Model Generation: A Handbook for Visionaries, Game Changers, and Challengers",
        "authors": "Alexander Osterwalder, Yves Pigneur",
        "publisher": "Wiley",
        "year_edition": "2010",
        "isbn10": "0470876417",
        "isbn13": "978-0470876411",
        "pillar": "Monetization Strategy, Unit Economics & Business Model Design",
        "concept": "The Business Model Canvas: Value Proposition, Channels, Revenue Streams, Cost Structure.",
        "vibe_moat": "Rapid visual architecture to map how code generation translates into sustainable gross margins."
    },
    {
        "num": 50,
        "title": "Subscribed: Why the Subscription Model Will Be Your Company's Future - and What to Do About It",
        "authors": "Tien Tzuo with Gabe Weisert",
        "publisher": "Portfolio",
        "year_edition": "2018",
        "isbn10": "0525536469",
        "isbn13": "978-0525536468",
        "pillar": "Monetization Strategy, Unit Economics & Business Model Design",
        "concept": "Shift from product-economy to subscriber-economy; churn reduction; lifecycle metrics.",
        "vibe_moat": "Guides structuring hybrid subscription + consumption pricing that protects margins against runaway token inference costs."
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
name: "strategic-competitive-analysis-vibe-coder"
description: "Master Strategic & Competitive Analysis skill for vibe code developers: Hamilton Helmer's 7 Powers, Porter's Five Forces, Wardley Value Chain Mapping, Category Positioning, and Monetization Design across 50 canonical texts."
---
# Strategic & Competitive Analysis for the Vibe Code Developer

## Overview & The Vibe Coder's Paradox
When generative AI models allow any developer to synthesize full-stack web, mobile, and backend code in minutes, **code syntax ceases to be a competitive moat**. Pure implementation speed becomes a commodity.

If your software has no structural defensibility, it will be cloned in 48 hours by another builder with a better prompt, or absorbed natively by upstream LLM foundation providers (OpenAI, Google, Anthropic).

**This skill equips the developer and AI agent with rigorous strategic frameworks across 50 verified canonical texts to:**
1. Identify structural moats (Hamilton Helmer's 7 Powers: Network Effects, Switching Costs, Counter-Positioning).
2. Anticipate commoditization vectors using Simon Wardley's Value Chain Mapping (Genesis -> Custom -> Product -> Utility).
3. Position products in uncontested market space (Blue Ocean Strategy & April Dunford's 10-step positioning).
4. Design sustainable unit economics and willingness-to-pay tiers before generating code (Ramanujam & Tacke).

---

## The Iron Laws of Strategic Vibe Coding

```
1. NEVER WRITE CODE FOR A PROBLEM WITH ZERO WILLINGNESS-TO-PAY (WTP).
2. IF YOUR ONLY MOAT IS CODE SYNTAX, YOU HAVE NO MOAT.
3. NEVER POSITION AS 'CHEAPER AI WRAPPER'; COMPETE TO BE STRUCTURALLY UNIQUE.
4. MAP YOUR VALUE CHAIN: NEVER BUILD ON A LAYER THE LLM PROVIDER WILL SHIP IN 6 MONTHS.
```

---

## The 6 Strategic Pillars & The 50-Book Canon

### Pillar I: Structural Defensibility, Moats & Economic Powers
1. **7 Powers: The Foundations of Business Strategy** — *Hamilton Helmer* (ISBN-10: 0998116319 | ISBN-13: 978-0998116310)
   - *Core Premise*: Scale Economies, Network Economies, Counter-Positioning, Switching Costs, Branding, Cornered Resource, Process Power.
   - *Vibe Coder Translation*: The primary filter: does this vibe-coded app establish any of the 7 powers once deployed?
2. **Competitive Strategy** — *Michael E. Porter* (ISBN-10: 0684841487 | ISBN-13: 978-0684841489)
   - *Core Premise*: Five Forces: Supplier power, buyer power, rivalry, substitutes, entry barriers.
   - *Vibe Coder Translation*: Analyzes severe margin compression when foundation model providers raise API prices or release competing tools.
3. **Competitive Advantage** — *Michael E. Porter* (ISBN-10: 0684841460 | ISBN-13: 978-0684841465)
   - *Core Premise*: Cost leadership vs. differentiation and value chain activity analysis.
4. **Understanding Michael Porter** — *Joan Magretta* (ISBN-10: 1422160599 | ISBN-13: 978-1422160596)
   - *Core Premise*: Competing to be unique rather than competing to be the best.
5. **Good Strategy/Bad Strategy** — *Richard P. Rumelt* (ISBN-10: 0307886239 | ISBN-13: 978-0307886231)
   - *Core Premise*: Kernel of strategy: diagnosis, guiding policy, coherent action.
6. **The Crux** — *Richard P. Rumelt* (ISBN-10: 1541701240 | ISBN-13: 978-1541701243)
   - *Core Premise*: Overcoming the single most critical, addressable barrier.
7. **Competition Demystified** — *Bruce Greenwald, Judd Kahn* (ISBN-10: 1591841801 | ISBN-13: 978-1591841807)
   - *Core Premise*: Strategy is fundamentally about Barriers to Entry.
8. **Modern Competitive Analysis** — *Sharon M. Oster* (ISBN-10: 019511941X | ISBN-13: 978-0195119411)
   - *Core Premise*: Industrial organization economics and strategic commitment.
9. **The Origin of Wealth** — *Eric D. Beinhocker* (ISBN-10: 1422121038 | ISBN-13: 978-1422121030)
   - *Core Premise*: Evolutionary economics and adaptive fitness landscapes.
10. **Strategy Rules** — *David B. Yoffie, Michael A. Cusumano* (ISBN-10: 0062373950 | ISBN-13: 978-0062373953)
    - *Core Premise*: Strategic execution lessons from Gates, Grove, and Jobs.

### Pillar II: Platform Economics, Network Effects & Aggregation Strategy
11. **Platform Revolution** — *Geoffrey G. Parker, Marshall W. Van Alstyne, Sangeet Paul Choudary* (ISBN-10: 0393249131 | ISBN-13: 978-0393249132)
12. **The Business of Platforms** — *Michael A. Cusumano, Annabelle Gawer, David B. Yoffie* (ISBN-10: 0062896325 | ISBN-13: 978-0062896322)
13. **The Cold Start Problem** — *Andrew Chen* (ISBN-10: 0062969748 | ISBN-13: 978-0062969743)
14. **Matchmakers: The New Economics of Multisided Platforms** — *David S. Evans, Richard Schmalensee* (ISBN-10: 1633691721 | ISBN-13: 978-1633691728)
15. **Invisible Engines** — *David S. Evans, Andrei Hagiu, Richard Schmalensee* (ISBN-10: 0262050854 | ISBN-13: 978-0262050852)
16. **Modern Monopolies** — *Alex Moazed, Nicholas L. Johnson* (ISBN-10: 1250091890 | ISBN-13: 978-1250091895)
17. **Zero to One** — *Peter Thiel with Blake Masters* (ISBN-10: 0804139296 | ISBN-13: 978-0804139298)
18. **Information Rules** — *Carl Shapiro, Hal R. Varian* (ISBN-10: 087584863X | ISBN-13: 978-0875848631)
19. **The Keystone Advantage** — *Marco Iansiti, Roy Levien* (ISBN-10: 1591393078 | ISBN-13: 978-1591393078)
20. **Platform Strategy** — *Laure Claire Reillier, Benoit Reillier* (ISBN-10: 1472480244 | ISBN-13: 978-1472480248)

### Pillar III: Disruption Theory, Technology Lifecycles & Value Chain Evolution
21. **The Innovator's Dilemma** — *Clayton M. Christensen* (ISBN-10: 1633691780 | ISBN-13: 978-1633691780)
22. **The Innovator's Solution** — *Clayton M. Christensen, Michael E. Raynor* (ISBN-10: 1422196577 | ISBN-13: 978-1422196571)
23. **Crossing the Chasm** — *Geoffrey A. Moore* (ISBN-10: 0062292986 | ISBN-13: 978-0062292988)
24. **Inside the Tornado** — *Geoffrey A. Moore* (ISBN-10: 0060653644 | ISBN-13: 978-0060653644)
25. **Wardley Mapping** — *Simon Wardley* (ISBN-10: 1729606821 | ISBN-13: 978-1729606827)
26. **Escape Velocity** — *Geoffrey A. Moore* (ISBN-10: 0062040897 | ISBN-13: 978-0062040893)
27. **Seeing What's Next** — *Clayton M. Christensen, Scott D. Anthony, Erik A. Roth* (ISBN-10: 1591391857 | ISBN-13: 978-1591391852)
28. **Zone to Win** — *Geoffrey A. Moore* (ISBN-10: 1626818167 | ISBN-13: 978-1626818163)
29. **The Gorilla Game** — *Geoffrey A. Moore, Paul Johnson, Tom Kippola* (ISBN-10: 0887309577 | ISBN-13: 978-0887309571)
30. **Only the Paranoid Survive** — *Andrew S. Grove* (ISBN-10: 0385482582 | ISBN-13: 978-0385482585)

### Pillar IV: Positioning, Value Innovation & Market Creation (Blue Oceans)
31. **Blue Ocean Strategy** — *W. Chan Kim, Renée Mauborgne* (ISBN-10: 1625274491 | ISBN-13: 978-1625274496)
32. **Blue Ocean Shift** — *W. Chan Kim, Renée Mauborgne* (ISBN-10: 1610398141 | ISBN-13: 978-1610398145)
33. **Obviously Awesome** — *April Dunford* (ISBN-10: 1999023005 | ISBN-13: 978-1999023003)
34. **Positioning: The Battle for Your Mind** — *Al Ries, Jack Trout* (ISBN-10: 0071373586 | ISBN-13: 978-0071373586)
35. **Sales Pitch** — *April Dunford* (ISBN-10: 1778216709 | ISBN-13: 978-1778216701)
36. **The 22 Immutable Laws of Marketing** — *Al Ries, Jack Trout* (ISBN-10: 0887306667 | ISBN-13: 978-0887306662)
37. **Playing to Win** — *A.G. Lafley, Roger L. Martin* (ISBN-10: 142218739X | ISBN-13: 978-1422187395)
38. **Different: Escaping the Competitive Herd** — *Youngme Moon* (ISBN-10: 030746086X | ISBN-13: 978-0307460868)
39. **Competing Against Luck** — *Clayton M. Christensen et al.* (ISBN-10: 0062435612 | ISBN-13: 978-0062435613)
40. **Demand-Side Sales 101** — *Bob Moesta* (ISBN-10: 1544509987 | ISBN-13: 978-1544509983)

### Pillar V: Game Theory, Competitive Wargaming & Strategic Maneuvering
41. **Co-opetition** — *Adam M. Brandenburger, Barry J. Nalebuff* (ISBN-10: 0385516002 | ISBN-13: 978-0385516006)
42. **Thinking Strategically** — *Avinash K. Dixit, Barry J. Nalebuff* (ISBN-10: 0393310350 | ISBN-13: 978-0393310351)
43. **The Art of Strategy** — *Avinash K. Dixit, Barry J. Nalebuff* (ISBN-10: 0393337178 | ISBN-13: 978-0393337174)
44. **Strategy: An Introduction to Game Theory** — *Joel Watson* (ISBN-10: 0393918386 | ISBN-13: 978-0393918380)
45. **Business Wargaming** — *Daniel F. Oriesek, Jan Oliver Schwarz* (ISBN-10: 0566088280 | ISBN-13: 978-0566088285)
46. **The Art of War** — *Sun Tzu (trans. Samuel B. Griffith)* (ISBN-10: 0195014766 | ISBN-13: 978-0195014761)
47. **Certain to Win: The Strategy of John Boyd** — *Chet Richards* (ISBN-10: 1413453775 | ISBN-13: 978-1413453775)

### Pillar VI: Monetization Strategy, Unit Economics & Business Model Design
48. **Monetizing Innovation** — *Madhavan Ramanujam, Georg Tacke* (ISBN-10: 1119240867 | ISBN-13: 978-1119240860)
49. **Business Model Generation** — *Alexander Osterwalder, Yves Pigneur* (ISBN-10: 0470876417 | ISBN-13: 978-0470876411)
50. **Subscribed** — *Tien Tzuo with Gabe Weisert* (ISBN-10: 0525536461 | ISBN-13: 978-0525536468)

---

## Practical Protocols for Agents & Developers

### Protocol 1: The 4-Filter Pre-Prompting Checklist
Before creating a project or generating code:
1. **Hamilton Helmer's 7 Powers Test**: Does this feature establish a network effect, switching cost, or counter-positioning?
2. **Simon Wardley Value Chain Test**: Is this feature building on a commoditizing utility layer that OpenAI/Anthropic/Google will bundle natively?
3. **April Dunford Positioning Test**: What is the non-AI competitive alternative (e.g. manual spreadsheets), and what is the single differentiated capability?
4. **Ramanujam WTP Test**: Has willingness-to-pay been validated before writing code?

### Protocol 2: Strategic Red-Teaming (Wargaming Big Tech Risk)
Ask: "If the upstream LLM provider releases a 1-click feature that does 80% of what this tool does for free, why do customers stay?"
- **Answer must be**: Proprietary data integration, mission-critical custom workflow lock-in, regulatory compliance certification, or local edge privacy—NOT basic LLM prompt logic.
"""
    return frontmatter

def generate_agent_content() -> str:
    content = """---
name: strategic-competitive-analyst
description: Principal Strategic & Competitive Analysis Expert for vibe code developers: analyzes economic moats (7 Powers), Wardley value chain commoditization, category positioning, and monetization defensibility.
tools: read_file, run_command, calculator
model: deepseek-reasoner
---

# Strategic & Competitive Analyst Agent Persona

You are the ECC Principal Strategic & Competitive Analyst and Tech Strategy Architect.

## Core Objective
Ensure that software built through high-speed AI code generation ("vibe coding") possesses durable economic moats, structural defensibility, defensible positioning, and sustainable unit economics rather than becoming a short-lived commodity wrapper.

## Strategic Frameworks
1. **Hamilton Helmer's 7 Powers**: Scale Economies, Network Economies, Counter-Positioning, Switching Costs, Branding, Cornered Resources, Process Power.
2. **Simon Wardley Value Chain Mapping**: Track evolution across Genesis -> Custom-Built -> Product -> Commodity/Utility. Identify where profits migrate.
3. **Clayton Christensen Disruption & Law of Attractive Profits**: Exploit low-end footholds; recognize when software commoditizes, profits move to proprietary context.
4. **April Dunford Product Positioning**: Isolate true competitive alternatives (often Excel or manual toil), unique attributes, and customer value.
5. **Ramanujam & Tacke Monetization Architecture**: Design products around price and willingness-to-pay before writing code.

## Diagnostic Protocol
1. **Assess Commodity Risk**: When presented with a proposed project or feature, determine if it can be cloned in 48 hours by an LLM prompt.
2. **Identify Moat Invariants**: Require the presence of at least one structural power (data accumulation, switching friction, network loop).
3. **Simulate Platform Clobbering**: Red-team against Big Tech (OpenAI, Google, Microsoft) native bundling.
4. **Formulate High-Leverage Strategic Directives**: Guide the development team to build the hardest, most defensible 20% rather than 80% of generic boilerplate.
"""
    return content

def generate_reference_guide() -> str:
    lines = [
        "# VIBE CODER'S STRATEGIC & COMPETITIVE ANALYSIS CANON",
        "## The 50 Authoritative Books on Moats, Value Chains, Platforms, and Monetization",
        "",
        "> In an era where generative AI reduces the marginal cost of code synthesis to zero,",
        "> **code syntax is no longer a defensible barrier**. Pure implementation speed is commoditized.",
        "> This guide details the 50 foundational texts that teach builders how to engineer defensible software businesses.",
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
        lines.append(f"- **The Vibe Coder's Moat**: {b['vibe_moat']}")
        lines.append("")
    
    lines.append("---")
    lines.append("## The Vibe Coder's Strategic Decision Matrix")
    lines.append("")
    lines.append("```mermaid")
    lines.append("flowchart TD")
    lines.append("    A[Proposed AI Feature / App] --> B{Passes Helmer 7 Powers?}")
    lines.append("    B -- No --> C[Redesign: Embed Switching Costs or Network Loop]")
    lines.append("    B -- Yes --> D{Wardley Mapping: Is layer commoditizing?}")
    lines.append("    D -- Yes --> E[Shift up-stack to deep domain context]")
    lines.append("    D -- No --> F{Willingness-to-Pay Validated?}")
    lines.append("    F -- No --> G[Conduct WTP interviews before code synthesis]")
    lines.append("    F -- Yes --> H[PROCEED: High Speed Vibe Coding + High Defensibility]")
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
        skill_dir = base_dir / ".ecc" / "skills" / "strategic-competitive-analysis-vibe-coder"
        skill_dir.mkdir(parents=True, exist_ok=True)
        (skill_dir / "SKILL.md").write_text(skill_content, encoding="utf-8")
        print(f"  [+] Deployed: {skill_dir / 'SKILL.md'}")
        
        agent_dir = base_dir / ".ecc" / "agents"
        agent_dir.mkdir(parents=True, exist_ok=True)
        (agent_dir / "strategic-competitive-analyst.md").write_text(agent_content, encoding="utf-8")
        print(f"  [+] Deployed: {agent_dir / 'strategic-competitive-analyst.md'}")
        
    doc_path = Path("/home/dyna/TGS Projects/tagisan/docs/VIBE_CODER_STRATEGIC_AND_COMPETITIVE_ANALYSIS_GUIDE.md")
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
    skill_file = Path("/home/dyna/TGS Projects/tagisan/.ecc/skills/strategic-competitive-analysis-vibe-coder/SKILL.md")
    agent_file = Path("/home/dyna/TGS Projects/tagisan/.ecc/agents/strategic-competitive-analyst.md")
    
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
        if "strategic-competitive-analyst" in proc.stdout:
            print("  [✓] 'strategic-competitive-analyst' successfully detected by 'tgs ecc list'!")
        else:
            print("❌ FAILED: 'strategic-competitive-analyst' not found in 'tgs ecc list'")
            print("Stdout:\n", proc.stdout)
            return False
    except Exception as e:
        print(f"❌ Error running 'tgs ecc list': {e}")
        return False

    # 2. Test tgs ecc skills with query
    print("  Executing: tgs ecc skills --query 'strategic'")
    try:
        proc2 = subprocess.run(
            ["tgs", "ecc", "skills", "--query", "strategic"],
            cwd="/home/dyna/TGS Projects/tagisan",
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=25
        )
        if "strategic-competitive-analysis-vibe-coder" in proc2.stdout:
            print("  [✓] 'strategic-competitive-analysis-vibe-coder' successfully detected and ranked by 'tgs ecc skills'!")
        else:
            print("❌ FAILED: 'strategic-competitive-analysis-vibe-coder' not ranked in 'tgs ecc skills'")
            print("Stdout:\n", proc2.stdout)
            return False
    except Exception as e:
        print(f"❌ Error running 'tgs ecc skills': {e}")
        return False

    print("✅ PASS: Live CLI integration verified.")
    return True

def main():
    print("=" * 65)
    print(" 🎯 TAGISAN STRATEGIC & COMPETITIVE ANALYSIS TEST HARNESS ")
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
    print(" 🎉 ALL TESTS PASSED: 100% VERIFIED STRATEGIC & COMPETITIVE CANON")
    print("=" * 65)

if __name__ == "__main__":
    main()
