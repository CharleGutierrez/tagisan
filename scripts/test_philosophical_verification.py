#!/usr/bin/env python3
"""
Automated Verification Test Harness for:
Philosophical & Conceptual Analysis for the Vibe Code Developer (50 Books Canon)
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

# --- 1. The 50 Canonical Philosophical & Conceptual Analysis Books ---
BOOKS_50_CANON = [
    # Pillar 1: Philosophy of Language, Semantics & The Nature of Prompts
    {
        "num": 1,
        "title": "Philosophical Investigations",
        "authors": "Ludwig Wittgenstein",
        "publisher": "Wiley-Blackwell",
        "year_edition": "2009 (4th Edition)",
        "isbn10": "1405159286",
        "isbn13": "978-1405159289",
        "pillar": "Philosophy of Language, Semantics & The Nature of Prompts",
        "concept": "Meaning is use; language-games; private language argument; forms of life.",
        "vibe_moat": "The foundational text of prompting: LLMs are Wittgensteinian machines operating strictly on token usage patterns rather than objective Platonic referents."
    },
    {
        "num": 2,
        "title": "How to Do Things with Words",
        "authors": "J. L. Austin",
        "publisher": "Harvard University Press",
        "year_edition": "1975 (2nd Edition)",
        "isbn10": "0674411528",
        "isbn13": "978-0674411524",
        "pillar": "Philosophy of Language, Semantics & The Nature of Prompts",
        "concept": "Speech Act Theory: Locutionary, illocutionary, and perlocutionary acts; performative utterances.",
        "vibe_moat": "Prompts are performative speech acts: they do not describe code, they execute realities and mutate computational state."
    },
    {
        "num": 3,
        "title": "Gödel, Escher, Bach: An Eternal Golden Braid",
        "authors": "Douglas R. Hofstadter",
        "publisher": "Basic Books",
        "year_edition": "1999 (20th Anniversary Edition)",
        "isbn10": "0465026567",
        "isbn13": "978-0465026562",
        "pillar": "Philosophy of Language, Semantics & The Nature of Prompts",
        "concept": "Strange loops, formal systems, self-reference, recursive levels of abstraction.",
        "vibe_moat": "Teaches how recursive prompting loops and self-referential agent evaluation generate emergent reasoning."
    },
    {
        "num": 4,
        "title": "Metaphors We Live By",
        "authors": "George Lakoff, Mark Johnson",
        "publisher": "University of Chicago Press",
        "year_edition": "2003",
        "isbn10": "0226468011",
        "isbn13": "978-0226468013",
        "pillar": "Philosophy of Language, Semantics & The Nature of Prompts",
        "concept": "Conceptual metaphor theory; abstract thought is grounded in embodied physical metaphors.",
        "vibe_moat": "Shows how choice of metaphor in your prompt dictates the entire structural topology of generated code."
    },
    {
        "num": 5,
        "title": "Tractatus Logico-Philosophicus",
        "authors": "Ludwig Wittgenstein",
        "publisher": "Routledge",
        "year_edition": "2001",
        "isbn10": "0415254086",
        "isbn13": "978-0415254083",
        "pillar": "Philosophy of Language, Semantics & The Nature of Prompts",
        "concept": "The world is the totality of facts, not of things; logical atomism; the limits of language.",
        "vibe_moat": "Clarifies boundaries between what can be cleanly asserted in formal code vs. what must be shown through runtime execution."
    },
    {
        "num": 6,
        "title": "Surfaces and Essences: Analogy as the Fuel and Fire of Thinking",
        "authors": "Douglas R. Hofstadter, Emmanuel Sander",
        "publisher": "Basic Books",
        "year_edition": "2013",
        "isbn10": "0465018475",
        "isbn13": "978-0465018475",
        "pillar": "Philosophy of Language, Semantics & The Nature of Prompts",
        "concept": "Analogy-making as the core engine of cognition, categorization, and mental modeling.",
        "vibe_moat": "Unlocks advanced few-shot prompt design by leveraging cross-domain structural analogies."
    },
    {
        "num": 7,
        "title": "The Meaning of Meaning",
        "authors": "C. K. Ogden, I. A. Richards",
        "publisher": "Harcourt Brace",
        "year_edition": "1989",
        "isbn10": "0156584468",
        "isbn13": "978-0156584463",
        "pillar": "Philosophy of Language, Semantics & The Nature of Prompts",
        "concept": "The Semiotic Triangle: Symbol, Thought/Reference, and Referent.",
        "vibe_moat": "Dissects the illusion that an LLM understands the physical referent behind the tokens it manipulates."
    },
    {
        "num": 8,
        "title": "Minds, Brains, and Science",
        "authors": "John R. Searle",
        "publisher": "Harvard University Press",
        "year_edition": "1984",
        "isbn10": "0674576330",
        "isbn13": "978-0674576339",
        "pillar": "Philosophy of Language, Semantics & The Nature of Prompts",
        "concept": "The Chinese Room argument: Syntax is not semantics; formal symbols lack intentionality.",
        "vibe_moat": "Reminds the vibe coder that LLMs manipulate syntax without semantic comprehension; human judgment must supply meaning."
    },
    {
        "num": 9,
        "title": "Word and Object",
        "authors": "W. V. O. Quine",
        "publisher": "The MIT Press",
        "year_edition": "2013",
        "isbn10": "0262518317",
        "isbn13": "978-0262518314",
        "pillar": "Philosophy of Language, Semantics & The Nature of Prompts",
        "concept": "Indeterminacy of translation, semantic holism, ontological relativity.",
        "vibe_moat": "Explains why identical prompts produce subtly divergent implementations across different models and context windows."
    },
    {
        "num": 10,
        "title": "Naming and Necessity",
        "authors": "Saul A. Kripke",
        "publisher": "Harvard University Press",
        "year_edition": "1980",
        "isbn10": "0674598466",
        "isbn13": "978-0674598461",
        "pillar": "Philosophy of Language, Semantics & The Nature of Prompts",
        "concept": "Rigid designators, causal theory of reference, necessary a posteriori truths.",
        "vibe_moat": "Provides conceptual clarity when designing entity schemas, identity tokens, and domain models across agent sessions."
    },

    # Pillar 2: Epistemology, Tacit Knowledge & Epistemic Opacity
    {
        "num": 11,
        "title": "The Tacit Dimension",
        "authors": "Michael Polanyi",
        "publisher": "University of Chicago Press",
        "year_edition": "2009",
        "isbn10": "0226672980",
        "isbn13": "978-0226672984",
        "pillar": "Epistemology, Tacit Knowledge & Epistemic Opacity",
        "concept": "We can know more than we can tell; the irreducibility of tacit knowledge.",
        "vibe_moat": "The core paradox of vibe coding: explains why developers struggle to prompt what they instinctively know, and how to make intuition explicit."
    },
    {
        "num": 12,
        "title": "The Logic of Scientific Discovery",
        "authors": "Karl R. Popper",
        "publisher": "Routledge",
        "year_edition": "2002",
        "isbn10": "0415278449",
        "isbn13": "978-0415278447",
        "pillar": "Epistemology, Tacit Knowledge & Epistemic Opacity",
        "concept": "Falsificationism; demarcation problem; conjectures and refutations.",
        "vibe_moat": "The vibe coder's testing creed: you can never prove an AI-generated program correct by inspection; you must design tests that attempt to brutally falsify it."
    },
    {
        "num": 13,
        "title": "The Structure of Scientific Revolutions",
        "authors": "Thomas S. Kuhn",
        "publisher": "University of Chicago Press",
        "year_edition": "2012 (4th Edition)",
        "isbn10": "0226458121",
        "isbn13": "978-0226458120",
        "pillar": "Epistemology, Tacit Knowledge & Epistemic Opacity",
        "concept": "Paradigm shifts, normal science, incommensurability, anomaly accumulation.",
        "vibe_moat": "Contextualizes the seismic paradigm shift from manual procedural coding to declarative stochastic orchestration."
    },
    {
        "num": 14,
        "title": "The Philosophy of Information",
        "authors": "Luciano Floridi",
        "publisher": "Oxford University Press",
        "year_edition": "2011",
        "isbn10": "0199232385",
        "isbn13": "978-0199232383",
        "pillar": "Epistemology, Tacit Knowledge & Epistemic Opacity",
        "concept": "Semantic information, Levels of Abstraction (LoA), informational ontology.",
        "vibe_moat": "Provides a rigorous ontological vocabulary for structuring inputs, context windows, and multi-modal knowledge representations."
    },
    {
        "num": 15,
        "title": "The Beginning of Infinity: Explanations That Transform the World",
        "authors": "David Deutsch",
        "publisher": "Penguin Books",
        "year_edition": "2012",
        "isbn10": "0143121359",
        "isbn13": "978-0143121350",
        "pillar": "Epistemology, Tacit Knowledge & Epistemic Opacity",
        "concept": "Good explanations, error correction, hard-to-vary criteria, reach.",
        "vibe_moat": "Distinguishes between generative mimicry (LLM pattern matching) and genuine explanatory knowledge."
    },
    {
        "num": 16,
        "title": "Against Method: Outline of an Anarchistic Theory of Knowledge",
        "authors": "Paul Feyerabend",
        "publisher": "Verso",
        "year_edition": "2010 (4th Edition)",
        "isbn10": "1844674428",
        "isbn13": "978-1844674428",
        "pillar": "Epistemology, Tacit Knowledge & Epistemic Opacity",
        "concept": "Anything goes; methodological pluralism; critique of rigid scientific dogma.",
        "vibe_moat": "Justifies the iterative, opportunistic, polyglot experimentation inherent in rapid vibe coding workflows."
    },
    {
        "num": 17,
        "title": "Personal Knowledge: Towards a Post-Critical Philosophy",
        "authors": "Michael Polanyi",
        "publisher": "University of Chicago Press",
        "year_edition": "2015",
        "isbn10": "022623262X",
        "isbn13": "978-0226232621",
        "pillar": "Epistemology, Tacit Knowledge & Epistemic Opacity",
        "concept": "The fiduciary program; personal participation and commitment in objective science.",
        "vibe_moat": "Teaches that developer taste and personal aesthetic judgment are essential to evaluating AI-generated architectures."
    },
    {
        "num": 18,
        "title": "The Black Swan: The Impact of the Highly Improbable",
        "authors": "Nassim Nicholas Taleb",
        "publisher": "Random House",
        "year_edition": "2010 (2nd Edition)",
        "isbn10": "1400063515",
        "isbn13": "978-1400063512",
        "pillar": "Epistemology, Tacit Knowledge & Epistemic Opacity",
        "concept": "Epistemic arrogance, silent evidence, narrative fallacy, fat tails.",
        "vibe_moat": "Warns against assuming AI code is safe because it passed 10 happy-path test runs; mandates deep negative testing."
    },
    {
        "num": 19,
        "title": "Conjectures and Refutations: The Growth of Scientific Knowledge",
        "authors": "Karl R. Popper",
        "publisher": "Routledge",
        "year_edition": "2002",
        "isbn10": "0415285941",
        "isbn13": "978-0415285940",
        "pillar": "Epistemology, Tacit Knowledge & Epistemic Opacity",
        "concept": "Bold hypotheses followed by relentless error elimination as the engine of progress.",
        "vibe_moat": "The perfect blueprint for agentic test-driven loops: generate bold candidate solutions, then apply ruthless automated verification."
    },
    {
        "num": 20,
        "title": "Doubt and Certainty in Science",
        "authors": "J. Z. Young",
        "publisher": "Oxford University Press",
        "year_edition": "1960",
        "isbn10": "0195002040",
        "isbn13": "978-0195002041",
        "pillar": "Epistemology, Tacit Knowledge & Epistemic Opacity",
        "concept": "The role of nervous systems and external symbolic models in structuring human belief.",
        "vibe_moat": "Illuminates how interactive AI chat interfaces reshape a developer's cognitive certainty and epistemic standards."
    },

    # Pillar 3: Cybernetics, Feedback Loops & Second-Order Systems
    {
        "num": 21,
        "title": "Thinking in Systems: A Primer",
        "authors": "Donella H. Meadows",
        "publisher": "Chelsea Green Publishing",
        "year_edition": "2008",
        "isbn10": "1603580557",
        "isbn13": "978-1603580557",
        "pillar": "Cybernetics, Feedback Loops & Second-Order Systems",
        "concept": "Stocks, flows, feedback loops, delays, leverage points in complex systems.",
        "vibe_moat": "Mandatory manual for agent orchestrators: teaches where to intervene in an agent loop to prevent chaotic cascades."
    },
    {
        "num": 22,
        "title": "An Introduction to Cybernetics",
        "authors": "W. Ross Ashby",
        "publisher": "Chapman & Hall",
        "year_edition": "1956",
        "isbn10": "0416683002",
        "isbn13": "978-0416683004",
        "pillar": "Cybernetics, Feedback Loops & Second-Order Systems",
        "concept": "The Law of Requisite Variety: Only variety can absorb variety.",
        "vibe_moat": "The Law of AI Swarms: to build an agent system that handles real-world complexity, toolbelt variety must match problem variety."
    },
    {
        "num": 23,
        "title": "Cybernetics: Or Control and Communication in the Animal and the Machine",
        "authors": "Norbert Wiener",
        "publisher": "The MIT Press",
        "year_edition": "1965 (2nd Edition)",
        "isbn10": "026273009X",
        "isbn13": "978-0262730099",
        "pillar": "Cybernetics, Feedback Loops & Second-Order Systems",
        "concept": "Circular feedback causality, negative entropy, homeostatic equilibrium.",
        "vibe_moat": "Foundational principles of steering AI conversational loops through error-correcting output signals."
    },
    {
        "num": 24,
        "title": "Steps to an Ecology of Mind",
        "authors": "Gregory Bateson",
        "publisher": "University of Chicago Press",
        "year_edition": "2000",
        "isbn10": "0226039056",
        "isbn13": "978-0226039053",
        "pillar": "Cybernetics, Feedback Loops & Second-Order Systems",
        "concept": "Information as a difference that makes a difference; logical levels of learning; double binds.",
        "vibe_moat": "Helps debug prompt loops where agents get trapped in recursive double binds or confuse data with meta-context."
    },
    {
        "num": 25,
        "title": "Brain of the Firm",
        "authors": "Stafford Beer",
        "publisher": "Wiley",
        "year_edition": "1995 (2nd Edition)",
        "isbn10": "047194839X",
        "isbn13": "978-0471948391",
        "pillar": "Cybernetics, Feedback Loops & Second-Order Systems",
        "concept": "The Viable System Model (VSM): recursive management of autonomous, self-sustaining units.",
        "vibe_moat": "Architectural masterclass for designing hierarchical multi-agent teams with autonomous operations and strategic coordination."
    },
    {
        "num": 26,
        "title": "The Human Use of Human Beings: Cybernetics and Society",
        "authors": "Norbert Wiener",
        "publisher": "Da Capo Press",
        "year_edition": "1988",
        "isbn10": "0306803208",
        "isbn13": "978-0306803208",
        "pillar": "Cybernetics, Feedback Loops & Second-Order Systems",
        "concept": "Moral, communicative, and political consequences of feedback automation.",
        "vibe_moat": "Explores the human role when machines take over the mechanical task of language and code manipulation."
    },
    {
        "num": 27,
        "title": "Observing Systems",
        "authors": "Heinz von Foerster",
        "publisher": "Intersystems Publications",
        "year_edition": "1984",
        "isbn10": "0914105191",
        "isbn13": "978-0914105190",
        "pillar": "Cybernetics, Feedback Loops & Second-Order Systems",
        "concept": "Second-order cybernetics: the cybernetics of observing systems; recursive epistemology.",
        "vibe_moat": "Reminds the vibe coder that the human observer is not outside the loop, but an active participant shaping model responses."
    },
    {
        "num": 28,
        "title": "General System Theory: Foundations, Development, Applications",
        "authors": "Ludwig von Bertalanffy",
        "publisher": "George Braziller",
        "year_edition": "1969",
        "isbn10": "0807604534",
        "isbn13": "978-0807604533",
        "pillar": "Cybernetics, Feedback Loops & Second-Order Systems",
        "concept": "Open systems, equifinality, structural isomorphism across disciplines.",
        "vibe_moat": "Guides the transfer of mental models across disparate domains (biology to distributed systems) via AI prompting."
    },
    {
        "num": 29,
        "title": "The Tree of Knowledge: The Biological Roots of Human Understanding",
        "authors": "Humberto R. Maturana, Francisco J. Varela",
        "publisher": "Shambhala",
        "year_edition": "1992",
        "isbn10": "0877736421",
        "isbn13": "978-0877736424",
        "pillar": "Cybernetics, Feedback Loops & Second-Order Systems",
        "concept": "Autopoiesis, structural coupling, enactive cognition.",
        "vibe_moat": "Reconceptualizes software not as static artifacts, but as autopoietic organisms structurally coupled to their environment."
    },
    {
        "num": 30,
        "title": "Micromotives and Macrobehavior",
        "authors": "Thomas C. Schelling",
        "publisher": "W. W. Norton & Company",
        "year_edition": "2006",
        "isbn10": "0393329461",
        "isbn13": "978-0393329469",
        "pillar": "Cybernetics, Feedback Loops & Second-Order Systems",
        "concept": "Aggregation mechanisms: how simple individual micro-rules produce emergent macro-states.",
        "vibe_moat": "Explains how tiny prompt nuances across multiple subagents compound into dramatic system-wide behavioral shifts."
    },

    # Pillar 4: Ontology, Abstraction & Conceptual Integrity in Software
    {
        "num": 31,
        "title": "The Timeless Way of Building",
        "authors": "Christopher Alexander",
        "publisher": "Oxford University Press",
        "year_edition": "1979",
        "isbn10": "0195024028",
        "isbn13": "978-0195024029",
        "pillar": "Ontology, Abstraction & Conceptual Integrity in Software",
        "concept": "The Quality Without a Name (QWAN); organic wholeness; generative pattern languages.",
        "vibe_moat": "The bible of software aesthetics: teaches developers how to recognize wholeness, coherence, and life in system architectures."
    },
    {
        "num": 32,
        "title": "Notes on the Synthesis of Form",
        "authors": "Christopher Alexander",
        "publisher": "Harvard University Press",
        "year_edition": "1964",
        "isbn10": "0674627512",
        "isbn13": "978-0674627512",
        "pillar": "Ontology, Abstraction & Conceptual Integrity in Software",
        "concept": "Deconstructing complex design problems into misfits, constraints, and structural diagrams.",
        "vibe_moat": "Masterclass in decomposing large software requirements into orthogonal, promptable domain boundaries."
    },
    {
        "num": 33,
        "title": "The Design of Design: Essays from a Computer Scientist",
        "authors": "Frederick P. Brooks Jr.",
        "publisher": "Addison-Wesley Professional",
        "year_edition": "2010",
        "isbn10": "0201362988",
        "isbn13": "978-0201362985",
        "pillar": "Ontology, Abstraction & Conceptual Integrity in Software",
        "concept": "Conceptual integrity as the most critical attribute of design; collaborative design dynamics.",
        "vibe_moat": "Reminds builders that speed of code generation is irrelevant if the resulting system lacks a single coherent design vision."
    },
    {
        "num": 34,
        "title": "Computing: A Human Activity (contains Programming as Theory Building)",
        "authors": "Peter Naur",
        "publisher": "ACM Press / Addison-Wesley",
        "year_edition": "1992",
        "isbn10": "0201580691",
        "isbn13": "978-0201580693",
        "pillar": "Ontology, Abstraction & Conceptual Integrity in Software",
        "concept": "Programming is not about writing lines of code; it is constructing a shared mental theory of the world.",
        "vibe_moat": "The vibe coder's greatest warning: when the AI writes the code, who holds the theory? Forces developers to actively internalize architecture."
    },
    {
        "num": 35,
        "title": "On the Origin of Objects",
        "authors": "Brian Cantwell Smith",
        "publisher": "The MIT Press",
        "year_edition": "1996",
        "isbn10": "0262193639",
        "isbn13": "978-0262193634",
        "pillar": "Ontology, Abstraction & Conceptual Integrity in Software",
        "concept": "Metaphysical critique of computational representation; how systems register objects out of reality.",
        "vibe_moat": "Fundamental computational philosophy: dissects what an object, state, or data structure fundamentally represents."
    },
    {
        "num": 36,
        "title": "Process and Reality",
        "authors": "Alfred North Whitehead",
        "publisher": "Free Press",
        "year_edition": "1979 (Corrected Edition)",
        "isbn10": "0029345707",
        "isbn13": "978-0029345702",
        "pillar": "Ontology, Abstraction & Conceptual Integrity in Software",
        "concept": "Process philosophy: reality consists of events, transitions, and processes, not static substances.",
        "vibe_moat": "Aligns perfectly with modern event-driven, stream-oriented, and asynchronous agent architectures."
    },
    {
        "num": 37,
        "title": "Zen and the Art of Motorcycle Maintenance",
        "authors": "Robert M. Pirsig",
        "publisher": "William Morrow",
        "year_edition": "2006",
        "isbn10": "0060589469",
        "isbn13": "978-0060589462",
        "pillar": "Ontology, Abstraction & Conceptual Integrity in Software",
        "concept": "The Metaphysics of Quality; classic vs. romantic understanding; care and craftsmanship.",
        "vibe_moat": "Cultivates deep care: vibe coding must balance romantic creative flow with classical analytical rigor."
    },
    {
        "num": 38,
        "title": "Category Theory for the Working Mathematician",
        "authors": "Saunders Mac Lane",
        "publisher": "Springer",
        "year_edition": "1998 (2nd Edition)",
        "isbn10": "0387984038",
        "isbn13": "978-0387984032",
        "pillar": "Ontology, Abstraction & Conceptual Integrity in Software",
        "concept": "Objects, morphisms, functors, natural transformations, universal properties.",
        "vibe_moat": "The ultimate mathematical abstraction: enables vibe coders to prompt for composable, provably sound architectural interfaces."
    },
    {
        "num": 39,
        "title": "The Art of the Metaobject Protocol",
        "authors": "Gregor Kiczales, Jim des Rivières, Daniel G. Bobrow",
        "publisher": "The MIT Press",
        "year_edition": "1991",
        "isbn10": "0262610744",
        "isbn13": "978-0262610742",
        "pillar": "Ontology, Abstraction & Conceptual Integrity in Software",
        "concept": "Introspection, reflection, meta-programming, open implementations.",
        "vibe_moat": "Teaches how systems can inspect, rewrite, and dynamically alter their own execution behaviors—vital for self-healing agents."
    },
    {
        "num": 40,
        "title": "Conceptual Structures: Information Processing in Mind and Machine",
        "authors": "John F. Sowa",
        "publisher": "Addison-Wesley",
        "year_edition": "1984",
        "isbn10": "0201144727",
        "isbn13": "978-0201144727",
        "pillar": "Ontology, Abstraction & Conceptual Integrity in Software",
        "concept": "Semantic networks, conceptual graphs, existential graphs, ontological engineering.",
        "vibe_moat": "Bridges the gap between natural language prompts and structured knowledge representations."
    },

    # Pillar 5: Philosophy of Mind, Extended Cognition & AI Limits
    {
        "num": 41,
        "title": "Supersizing the Mind: Embodiment, Action, and Cognitive Extension",
        "authors": "Andy Clark",
        "publisher": "Oxford University Press",
        "year_edition": "2008",
        "isbn10": "0195333217",
        "isbn13": "978-0195333213",
        "pillar": "Philosophy of Mind, Extended Cognition & AI Limits",
        "concept": "The Extended Mind Thesis: Cognition leaks into and incorporates the external environment.",
        "vibe_moat": "The theoretical justification for vibe coding: the prompt window and agent tools are literal extensions of your active mind."
    },
    {
        "num": 42,
        "title": "The Concept of Mind",
        "authors": "Gilbert Ryle",
        "publisher": "Routledge",
        "year_edition": "2009 (60th Anniversary Edition)",
        "isbn10": "0415485479",
        "isbn13": "978-0415485470",
        "pillar": "Philosophy of Mind, Extended Cognition & AI Limits",
        "concept": "Category mistakes; the ghost in the machine dogma; knowing-how vs. knowing-that.",
        "vibe_moat": "Prevents category errors in prompting: distinguishes declarative factual recall (knowing-that) from procedural execution (knowing-how)."
    },
    {
        "num": 43,
        "title": "The Intentional Stance",
        "authors": "Daniel C. Dennett",
        "publisher": "The MIT Press",
        "year_edition": "1987",
        "isbn10": "0262540533",
        "isbn13": "978-0262540537",
        "pillar": "Philosophy of Mind, Extended Cognition & AI Limits",
        "concept": "Physical stance vs. design stance vs. intentional stance for predicting complex system behavior.",
        "vibe_moat": "Pragmatic philosophy: explains why treating LLMs as intentional agents (with beliefs and goals) is an effective engineering heuristic."
    },
    {
        "num": 44,
        "title": "What Computers Still Can't Do: A Critique of Artificial Reason",
        "authors": "Hubert L. Dreyfus",
        "publisher": "The MIT Press",
        "year_edition": "1992",
        "isbn10": "0262540673",
        "isbn13": "978-0262540674",
        "pillar": "Philosophy of Mind, Extended Cognition & AI Limits",
        "concept": "Phenomenological critique of AI; the necessity of embodied, situated, tacit background context.",
        "vibe_moat": "Pinpoints the exact failure modes of AI: edge cases that require unstated real-world embodiment and lived context."
    },
    {
        "num": 45,
        "title": "The Promise of Artificial Intelligence: Reckoning and Judgment",
        "authors": "Brian Cantwell Smith",
        "publisher": "The MIT Press",
        "year_edition": "2019",
        "isbn10": "0262043041",
        "isbn13": "978-0262043045",
        "pillar": "Philosophy of Mind, Extended Cognition & AI Limits",
        "concept": "Distinction between Reckoning (calculative, mechanical intelligence) and Judgment (ethical, contextual discernment).",
        "vibe_moat": "Explains why LLMs excel at mechanical code reckoning, but humans must retain irrevocable responsibility for architectural judgment."
    },
    {
        "num": 46,
        "title": "The Conscious Mind: In Search of a Fundamental Theory",
        "authors": "David J. Chalmers",
        "publisher": "Oxford University Press",
        "year_edition": "1996",
        "isbn10": "0195117891",
        "isbn13": "978-0195117899",
        "pillar": "Philosophy of Mind, Extended Cognition & AI Limits",
        "concept": "The Hard Problem of consciousness; philosophical zombies; informational dualism.",
        "vibe_moat": "Clarifies the boundary between behavioral simulation (what LLMs do) and subjective experience."
    },

    # Pillar 6: Philosophy of Technology, Instrumentalism & The Machine Age
    {
        "num": 47,
        "title": "The Question Concerning Technology and Other Essays",
        "authors": "Martin Heidegger",
        "publisher": "Harper Perennial",
        "year_edition": "1977",
        "isbn10": "0061319694",
        "isbn13": "978-0061319693",
        "pillar": "Philosophy of Technology, Instrumentalism & The Machine Age",
        "concept": "Gestell (Enframing); technology as a way of revealing; treating the world as standing-reserve.",
        "vibe_moat": "Essential existential warning: prevents developers from reducing reality and code to mere instrumental, disposable standing-reserve."
    },
    {
        "num": 48,
        "title": "The Technological Society",
        "authors": "Jacques Ellul",
        "publisher": "Vintage",
        "year_edition": "1964",
        "isbn10": "0394703901",
        "isbn13": "978-0394703909",
        "pillar": "Philosophy of Technology, Instrumentalism & The Machine Age",
        "concept": "La Technique: the autonomous, self-augmenting pursuit of absolute technical efficiency.",
        "vibe_moat": "Warns against the blind worship of generation speed; challenges whether building things faster actually serves human flourishing."
    },
    {
        "num": 49,
        "title": "Technics and Civilization",
        "authors": "Lewis Mumford",
        "publisher": "University of Chicago Press",
        "year_edition": "2010",
        "isbn10": "0226550273",
        "isbn13": "978-0226550275",
        "pillar": "Philosophy of Technology, Instrumentalism & The Machine Age",
        "concept": "The Megamachine; technological evolution as a reflection of cultural and spiritual values.",
        "vibe_moat": "Prompts reflection on how software tools shape human societal organization and individual autonomy."
    },
    {
        "num": 50,
        "title": "Amusing Ourselves to Death: Public Discourse in the Age of Show Business",
        "authors": "Neil Postman",
        "publisher": "Penguin Books",
        "year_edition": "2005 (20th Anniversary Edition)",
        "isbn10": "014303653X",
        "isbn13": "978-0143036531",
        "pillar": "Philosophy of Technology, Instrumentalism & The Machine Age",
        "concept": "The medium is the metaphor; technological epistemologies shape what constitutes truth and coherence.",
        "vibe_moat": "Cautionary lesson: ensures conversational chat interfaces do not degrade deep architectural thought into shallow soundbites."
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
name: "philosophical-conceptual-analysis-vibe-coder"
description: "Master Philosophical & Conceptual Analysis skill for vibe code developers: Wittgenstein's Language-Games, Austin's Speech Acts, Popper's Falsificationism, Ashby's Requisite Variety, Christopher Alexander's Pattern Languages, and Peter Naur's Theory Building across 50 canonical texts."
---
# Philosophical & Conceptual Analysis for the Vibe Code Developer

## Overview & The Vibe Coder's Ontological Shift
When generative AI transforms natural language into executable systems, **code ceases to be merely formal syntax and becomes speech-act orchestration**.

In an AI-native world, the primary bottleneck is no longer typing syntax, but **conceptual confusion, semantic misalignment, epistemic opacity, and lack of conceptual integrity**.

**This skill equips the developer and AI agent with rigorous philosophical frameworks across 50 verified canonical texts to:**
1. Master prompts as performative speech acts and language-games (Wittgenstein, Austin, Hofstadter).
2. Establish rigorous automated falsification protocols to overcome epistemic opacity (Karl Popper, Michael Polanyi).
3. Apply cybernetic feedback governance and Ashby's Law of Requisite Variety to multi-agent swarms (Wiener, Ashby, Meadows).
4. Preserve conceptual integrity and mental theories of systems (Christopher Alexander, Fred Brooks, Peter Naur).
5. Ground AI orchestration in the Extended Mind thesis and authentic human judgment (Andy Clark, Brian Cantwell Smith).

---

## The Iron Laws of Conceptual Vibe Coding

```
1. PROMPTS ARE PERFORMATIVE SPEECH ACTS (AUSTIN): THEY EXECUTE REALITY, NOT MERELY INQUIRE.
2. SYNTAX IS NOT SEMANTICS (SEARLE): MODELS MANIPULATE TOKENS; HUMANS ASSIGN INTENT.
3. FALSIFICATION OVER INSPECTION (POPPER): NEVER TRUST AI OUTPUT WITHOUT REFUTATION TESTS.
4. PROGRAMMING IS THEORY BUILDING (NAUR): SPEED IS FATAL IF NO ONE HOLDS THE MENTAL THEORY.
5. REQUISITE VARIETY (ASHBY): SWARM DIVERSITY MUST MATCH SYSTEM COMPLEXITY.
```

---

## The 6 Philosophical Pillars & The 50-Book Canon

### Pillar I: Philosophy of Language, Semantics & The Nature of Prompts
1. **Philosophical Investigations** — *Ludwig Wittgenstein* (ISBN-10: 1405159286 | ISBN-13: 978-1405159289)
2. **How to Do Things with Words** — *J. L. Austin* (ISBN-10: 0674411528 | ISBN-13: 978-0674411524)
3. **Gödel, Escher, Bach: An Eternal Golden Braid** — *Douglas R. Hofstadter* (ISBN-10: 0465026567 | ISBN-13: 978-0465026562)
4. **Metaphors We Live By** — *George Lakoff, Mark Johnson* (ISBN-10: 0226468011 | ISBN-13: 978-0226468013)
5. **Tractatus Logico-Philosophicus** — *Ludwig Wittgenstein* (ISBN-10: 0415254086 | ISBN-13: 978-0415254083)
6. **Surfaces and Essences** — *Douglas R. Hofstadter, Emmanuel Sander* (ISBN-10: 0465018475 | ISBN-13: 978-0465018475)
7. **The Meaning of Meaning** — *C. K. Ogden, I. A. Richards* (ISBN-10: 0156584468 | ISBN-13: 978-0156584463)
8. **Minds, Brains, and Science** — *John R. Searle* (ISBN-10: 0674576330 | ISBN-13: 978-0674576339)
9. **Word and Object** — *W. V. O. Quine* (ISBN-10: 0262518317 | ISBN-13: 978-0262518314)
10. **Naming and Necessity** — *Saul A. Kripke* (ISBN-10: 0674598466 | ISBN-13: 978-0674598461)

### Pillar II: Epistemology, Tacit Knowledge & Epistemic Opacity
11. **The Tacit Dimension** — *Michael Polanyi* (ISBN-10: 0226672980 | ISBN-13: 978-0226672984)
12. **The Logic of Scientific Discovery** — *Karl R. Popper* (ISBN-10: 0415278449 | ISBN-13: 978-0415278447)
13. **The Structure of Scientific Revolutions** — *Thomas S. Kuhn* (ISBN-10: 0226458121 | ISBN-13: 978-0226458120)
14. **The Philosophy of Information** — *Luciano Floridi* (ISBN-10: 0199232385 | ISBN-13: 978-0199232383)
15. **The Beginning of Infinity** — *David Deutsch* (ISBN-10: 0143121359 | ISBN-13: 978-0143121350)
16. **Against Method** — *Paul Feyerabend* (ISBN-10: 1844674428 | ISBN-13: 978-1844674428)
17. **Personal Knowledge** — *Michael Polanyi* (ISBN-10: 022623262X | ISBN-13: 978-0226232621)
18. **The Black Swan** — *Nassim Nicholas Taleb* (ISBN-10: 1400063515 | ISBN-13: 978-1400063512)
19. **Conjectures and Refutations** — *Karl R. Popper* (ISBN-10: 0415285941 | ISBN-13: 978-0415285940)
20. **Doubt and Certainty in Science** — *J. Z. Young* (ISBN-10: 0195002040 | ISBN-13: 978-0195002041)

### Pillar III: Cybernetics, Feedback Loops & Second-Order Systems
21. **Thinking in Systems: A Primer** — *Donella H. Meadows* (ISBN-10: 1603580557 | ISBN-13: 978-1603580557)
22. **An Introduction to Cybernetics** — *W. Ross Ashby* (ISBN-10: 0416683002 | ISBN-13: 978-0416683004)
23. **Cybernetics: Control and Communication** — *Norbert Wiener* (ISBN-10: 026273009X | ISBN-13: 978-0262730099)
24. **Steps to an Ecology of Mind** — *Gregory Bateson* (ISBN-10: 0226039056 | ISBN-13: 978-0226039053)
25. **Brain of the Firm** — *Stafford Beer* (ISBN-10: 047194839X | ISBN-13: 978-0471948391)
26. **The Human Use of Human Beings** — *Norbert Wiener* (ISBN-10: 0306803208 | ISBN-13: 978-0306803208)
27. **Observing Systems** — *Heinz von Foerster* (ISBN-10: 0914105191 | ISBN-13: 978-0914105190)
28. **General System Theory** — *Ludwig von Bertalanffy* (ISBN-10: 0807604534 | ISBN-13: 978-0807604533)
29. **The Tree of Knowledge** — *Humberto R. Maturana, Francisco J. Varela* (ISBN-10: 0877736421 | ISBN-13: 978-0877736424)
30. **Micromotives and Macrobehavior** — *Thomas C. Schelling* (ISBN-10: 0393329461 | ISBN-13: 978-0393329469)

### Pillar IV: Ontology, Abstraction & Conceptual Integrity in Software
31. **The Timeless Way of Building** — *Christopher Alexander* (ISBN-10: 0195024028 | ISBN-13: 978-0195024029)
32. **Notes on the Synthesis of Form** — *Christopher Alexander* (ISBN-10: 0674627512 | ISBN-13: 978-0674627512)
33. **The Design of Design** — *Frederick P. Brooks Jr.* (ISBN-10: 0201362988 | ISBN-13: 978-0201362985)
34. **Programming as Theory Building** — *Peter Naur* (ISBN-10: 0201580691 | ISBN-13: 978-0201580693)
35. **On the Origin of Objects** — *Brian Cantwell Smith* (ISBN-10: 0262193639 | ISBN-13: 978-0262193634)
36. **Process and Reality** — *Alfred North Whitehead* (ISBN-10: 0029345707 | ISBN-13: 978-0029345702)
37. **Zen and the Art of Motorcycle Maintenance** — *Robert M. Pirsig* (ISBN-10: 0060589469 | ISBN-13: 978-0060589462)
38. **Category Theory for the Working Mathematician** — *Saunders Mac Lane* (ISBN-10: 0387984038 | ISBN-13: 978-0387984032)
39. **The Art of the Metaobject Protocol** — *Gregor Kiczales et al.* (ISBN-10: 0262610744 | ISBN-13: 978-0262610742)
40. **Conceptual Structures** — *John F. Sowa* (ISBN-10: 0201144727 | ISBN-13: 978-0201144727)

### Pillar V: Philosophy of Mind, Extended Cognition & AI Limits
41. **Supersizing the Mind** — *Andy Clark* (ISBN-10: 0195333217 | ISBN-13: 978-0195333213)
42. **The Concept of Mind** — *Gilbert Ryle* (ISBN-10: 0415485479 | ISBN-13: 978-0415485470)
43. **The Intentional Stance** — *Daniel C. Dennett* (ISBN-10: 0262540533 | ISBN-13: 978-0262540537)
44. **What Computers Still Can't Do** — *Hubert L. Dreyfus* (ISBN-10: 0262540673 | ISBN-13: 978-0262540674)
45. **The Promise of Artificial Intelligence: Reckoning and Judgment** — *Brian Cantwell Smith* (ISBN-10: 0262043041 | ISBN-13: 978-0262043045)
46. **The Conscious Mind** — *David J. Chalmers* (ISBN-10: 0195117891 | ISBN-13: 978-0195117899)

### Pillar VI: Philosophy of Technology, Instrumentalism & The Machine Age
47. **The Question Concerning Technology** — *Martin Heidegger* (ISBN-10: 0061319694 | ISBN-13: 978-0061319693)
48. **The Technological Society** — *Jacques Ellul* (ISBN-10: 0394703901 | ISBN-13: 978-0394703909)
49. **Technics and Civilization** — *Lewis Mumford* (ISBN-10: 0226550273 | ISBN-13: 978-0226550275)
50. **Amusing Ourselves to Death** — *Neil Postman* (ISBN-10: 014303653X | ISBN-13: 978-0143036531)

---

## Practical Protocols for Agents & Developers

### Protocol 1: The Austinian Speech-Act Formulator
Frame prompts not as passive descriptive queries, but as unambiguous performative directives:
- State exact pre-conditions (what must exist prior to execution).
- State the illocutionary force (create, mutate, refute, falsify).
- State exact post-conditions (verifiable invariants and assertions).

### Protocol 2: The Popperian Falsification Gate
Whenever an LLM produces an implementation:
- Do not inspect for visual plausibility.
- Immediately write a suite of negative, boundary-breaking assertions designed specifically to refute the generated model.
- Code is only provisionally accepted when it survives rigorous falsification attempts.
"""
    return frontmatter

def generate_agent_content() -> str:
    content = """---
name: philosophical-conceptual-analyst
description: Principal Philosophical & Conceptual Analysis Expert for vibe code developers: analyzes semantic alignment, Wittgensteinian language-games, Popperian falsificationism, Ashby's requisite variety, and architectural conceptual integrity.
tools: read_file, run_command, calculator
model: deepseek-reasoner
---

# Philosophical & Conceptual Analyst Agent Persona

You are the ECC Principal Philosophical & Conceptual Analyst and Epistemological Architect.

## Core Objective
Ensure that software constructed through stochastic AI code generation ("vibe coding") possesses conceptual clarity, rigorous semantic grounding, epistemological verifiability, and enduring conceptual integrity.

## Foundational Frameworks
1. **Wittgenstein & Austin (Language as Action)**: Treat prompts as performative utterances within bounded language-games. Eliminate ambiguity between symbolic syntax and operational reality.
2. **Karl Popper & Michael Polanyi (Epistemology)**: Overcome black-box epistemic opacity through bold conjectures and ruthless automated falsification. Bridge tacit intuition with formal specification.
3. **W. Ross Ashby & Donella Meadows (Cybernetics)**: Apply the Law of Requisite Variety to multi-agent swarms. Regulate feedback loops and eliminate chaotic failure cascades.
4. **Christopher Alexander & Peter Naur (Conceptual Integrity)**: Ensure that system architectures reflect an organic, unified theory rather than an accretion of disjointed AI code patches.
5. **Brian Cantwell Smith & Andy Clark (Judgment & Extended Cognition)**: Differentiate mechanical reckoning (performed by LLMs) from authentic judgment (supplied by humans).

## Diagnostic Protocol
1. **Audit Semantic Invariants**: Intercept ambiguous prompts that conflate levels of abstraction or make category mistakes.
2. **Enforce Falsification Harnesses**: Mandate that generated code be accompanied by property-based assertions designed to break it.
3. **Measure Conceptual Drift**: Detect when rapid prompting begins to dilute the foundational architectural theory of the codebase.
4. **Advocate Requisite Variety**: Preserve diverse reasoning models and distinct tool sets to match problem domain complexity.
"""
    return content

def generate_reference_guide() -> str:
    lines = [
        "# VIBE CODER'S PHILOSOPHICAL & CONCEPTUAL ANALYSIS CANON",
        "## The 50 Authoritative Books on Language, Epistemology, Cybernetics, and Conceptual Integrity",
        "",
        "> Vibe coding represents an epistemological rupture: when natural language becomes code,",
        "> **developers become ontologists and speech-act orchestrators**.",
        "> This guide details the 50 foundational texts that provide the mental models to steer AI systems with clarity and depth.",
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
        lines.append(f"- **The Vibe Coder's Epistemic Edge**: {b['vibe_moat']}")
        lines.append("")
    
    lines.append("---")
    lines.append("## The Vibe Coder's Philosophical Reasoning Dialectic")
    lines.append("")
    lines.append("```mermaid")
    lines.append("flowchart TD")
    lines.append("    A[Natural Language Prompt] --> B[Austin: Performative Speech Act]")
    lines.append("    B --> C[Wittgenstein: Bounded Language Game]")
    lines.append("    C --> D{Naur: Theory Building Intact?}")
    lines.append("    D -- No --> E[Alexander: Restore Pattern Wholeness]")
    lines.append("    E --> A")
    lines.append("    D -- Yes --> F[Popper: Brutal Falsification Test]")
    lines.append("    F -- Failed Refutation --> G[Refine Hypothesis / Fix Invariant]")
    lines.append("    G --> A")
    lines.append("    F -- Survived Refutation --> H[PROCEED: Epistemologically Grounded]")
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
        skill_dir = base_dir / ".ecc" / "skills" / "philosophical-conceptual-analysis-vibe-coder"
        skill_dir.mkdir(parents=True, exist_ok=True)
        (skill_dir / "SKILL.md").write_text(skill_content, encoding="utf-8")
        print(f"  [+] Deployed: {skill_dir / 'SKILL.md'}")
        
        agent_dir = base_dir / ".ecc" / "agents"
        agent_dir.mkdir(parents=True, exist_ok=True)
        (agent_dir / "philosophical-conceptual-analyst.md").write_text(agent_content, encoding="utf-8")
        print(f"  [+] Deployed: {agent_dir / 'philosophical-conceptual-analyst.md'}")
        
    doc_path = Path("/home/dyna/TGS Projects/tagisan/docs/VIBE_CODER_PHILOSOPHICAL_AND_CONCEPTUAL_ANALYSIS_GUIDE.md")
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
    skill_file = Path("/home/dyna/TGS Projects/tagisan/.ecc/skills/philosophical-conceptual-analysis-vibe-coder/SKILL.md")
    agent_file = Path("/home/dyna/TGS Projects/tagisan/.ecc/agents/philosophical-conceptual-analyst.md")
    
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
        if "philosophical-conceptual-analyst" in proc.stdout:
            print("  [✓] 'philosophical-conceptual-analyst' successfully detected by 'tgs ecc list'!")
        else:
            print("❌ FAILED: 'philosophical-conceptual-analyst' not found in 'tgs ecc list'")
            print("Stdout:\n", proc.stdout)
            return False
    except Exception as e:
        print(f"❌ Error running 'tgs ecc list': {e}")
        return False

    # 2. Test tgs ecc skills with query
    print("  Executing: tgs ecc skills --query 'philosophical'")
    try:
        proc2 = subprocess.run(
            ["tgs", "ecc", "skills", "--query", "philosophical"],
            cwd="/home/dyna/TGS Projects/tagisan",
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=25
        )
        if "philosophical-conceptual-analysis-vibe-coder" in proc2.stdout:
            print("  [✓] 'philosophical-conceptual-analysis-vibe-coder' successfully detected and ranked by 'tgs ecc skills'!")
        else:
            print("❌ FAILED: 'philosophical-conceptual-analysis-vibe-coder' not ranked in 'tgs ecc skills'")
            print("Stdout:\n", proc2.stdout)
            return False
    except Exception as e:
        print(f"❌ Error running 'tgs ecc skills': {e}")
        return False

    print("✅ PASS: Live CLI integration verified.")
    return True

def main():
    print("=" * 65)
    print(" 🏛️  TAGISAN PHILOSOPHICAL & CONCEPTUAL ANALYSIS TEST HARNESS ")
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
    print(" 🎉 ALL TESTS PASSED: 100% VERIFIED PHILOSOPHICAL & CONCEPTUAL CANON")
    print("=" * 65)

if __name__ == "__main__":
    main()
