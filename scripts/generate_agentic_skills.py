#!/usr/bin/env python3
"""
scripts/generate_agentic_skills.py

Generates all 100 Agentic Engineering and Vibe Code Development skill packages
under .ecc/skills/agentic-*/SKILL.md with full YAML frontmatter and comprehensive
engineering instructions.
"""

import os
import sys
from pathlib import Path

# Base directory for skills
BASE_DIR = Path(__file__).resolve().parent.parent
SKILLS_DIR = BASE_DIR / ".ecc" / "skills"

# Definitions of all 100 Books for Agentic Engineering & Vibe Coding
SKILLS = [
    # =========================================================================
    # Cluster 1: Multi-Agent Systems, Swarms & Coordination (1-10)
    # =========================================================================
    {
        "id": "agentic-wooldridge-multiagent-systems",
        "book": "An Introduction to MultiAgent Systems (2nd ed) - Michael Wooldridge",
        "desc": "BDI (Belief-Desire-Intention) agent architecture, FIPA-ACL communicative acts, contract net protocol, coalition formation, and multi-agent coordination for autonomous software engineering swarms.",
        "triggers": ["wooldridge", "multiagent-systems", "bdi-architecture", "belief-desire-intention", "fipa-acl", "contract-net-protocol", "agent-negotiation", "swarm-coordination"],
        "foundations": [
            "BDI Architecture: Beliefs represent epistemic state, Desires represent motivational goals, and Intentions represent committed computational action plans.",
            "Speech Act Theory & FIPA-ACL: Every message between agents MUST define performative acts (request, propose, accept-proposal, reject-proposal, inform) with formal pre- and post-conditions.",
            "Contract Net Protocol (CNP): Task allocation proceeds through Announcement -> Bidding -> Awarding -> Execution -> Result Reporting.",
        ],
        "protocol": "Implement autonomous multi-agent systems using explicit BDI state loops. Decouple agent communication via strongly typed ACL protocols. Structure task distribution using Contract Net Protocol with timeout guarantees and fallback bids.",
        "anti_patterns": [
            "Free-form unstructured chatter between agents leading to unbounded conversational divergence.",
            "Conflating desires (potential goals) with intentions (committed executable tasks), causing thrashing.",
        ],
    },
    {
        "id": "agentic-shoham-multiagent-foundations",
        "book": "Multiagent Systems: Algorithmic, Game-Theoretic, and Logical Foundations - Yoav Shoham & Kevin Leyton-Brown",
        "desc": "Game-theoretic equilibria, Nash equilibrium, mechanism design, distributed constraint satisfaction (DisCSP), and social choice rules for multi-agent resource allocation.",
        "triggers": ["shoham", "leyton-brown", "game-theory", "nash-equilibrium", "discsp", "mechanism-design", "social-choice", "distributed-constraint"],
        "foundations": [
            "Nash Equilibrium: Strategy profile where no agent has incentive to unilaterally deviate given others' strategies: u_i(s_i^*, s_{-i}^*) >= u_i(s_i, s_{-i}^*).",
            "Distributed Constraint Satisfaction (DisCSP): Agents solve local constraints while communicating state via Asynchronous Backtracking (ABT) or Asynchronous Weak-Commitment (AWC).",
            "Vickrey-Clarke-Groves (VCG) Mechanism: Truthful dominant-strategy mechanism design aligning private incentives with global system utility.",
        ],
        "protocol": "Structure multi-agent resource allocation and arbitration using game-theoretic mechanism design. Enforce incentive compatibility so truthful reporting is the dominant strategy. Solve distributed resource conflicts via DisCSP constraint propagation.",
        "anti_patterns": [
            "Assuming cooperative agents without aligning individual utility functions, inviting tragedy of the commons.",
            "Naive priority-based arbitration prone to starvation and cyclic bidding wars.",
        ],
    },
    {
        "id": "agentic-weiss-distributed-ai",
        "book": "Multiagent Systems: A Modern Approach to Distributed Artificial Intelligence - Gerhard Weiss",
        "desc": "Distributed problem solving, multi-agent reinforcement learning, blackboard architectures, decentralized task allocation, and organizational structures for cooperative agents.",
        "triggers": ["weiss", "distributed-ai", "blackboard-architecture", "distributed-problem-solving", "cooperative-agents", "coalition-formation"],
        "foundations": [
            "Blackboard Architecture: Knowledge sources post hypotheses, partial solutions, and activations to a shared structured blackboard managed by a controller.",
            "Multi-Agent Reinforcement Learning (MARL): Agents learn concurrently in non-stationary environments using value-function approximation or actor-critic policies.",
            "Organizational Topologies: Hierarchy, flat market, federation, and holonic structures governing agent authority, scope, and communication channels.",
        ],
        "protocol": "Design complex distributed problem solvers using a blackboard pattern for decoupled knowledge fusion. Define explicit organizational boundaries and escalation paths across specialist subagents.",
        "anti_patterns": [
            "Monolithic agent controllers creating single points of failure and throughput bottlenecks.",
            "Shared blackboard state without optimistic locking or transactional concurrency guards.",
        ],
    },
    {
        "id": "agentic-ferber-reactive-agents",
        "book": "Multi-Agent Systems: An Introduction to Distributed Artificial Intelligence - Jacques Ferber",
        "desc": "Situated and reactive agents, stimulus-response architectures, subsumption hierarchy, environmental affordances, and emergence in physical and virtual spaces.",
        "triggers": ["ferber", "reactive-agents", "subsumption-architecture", "situated-agents", "stimulus-response", "environmental-affordances"],
        "foundations": [
            "Stimulus-Response Invariant: Action a = f(s) is computed directly from sensor observation s without complex intermediate deliberative planning.",
            "Subsumption Layering: Higher-level competence layers subsume (suppress or inhibit) lower-level reactive behaviors without modifying lower layers.",
            "Affordance Landscape: The environment encodes cues that directly trigger agent actions, minimizing internal state overhead.",
        ],
        "protocol": "Build ultra-low-latency agents using layered stimulus-response architectures. Use subsumption to handle reflex responses (e.g. rate limit backoff, syntax error retries) while higher layers execute strategic flows.",
        "anti_patterns": [
            "Over-engineering simple reactive tasks into multi-turn deliberative LLM chains.",
            "Cyclic subsumption inhibition loops resulting in behavioral freeze.",
        ],
    },
    {
        "id": "agentic-kennedy-swarm-intelligence",
        "book": "Swarm Intelligence - James Kennedy & Russell Eberhart",
        "desc": "Particle Swarm Optimization (PSO), socio-cognitive velocity updates, inertia weight, cognitive vs social components, and fitness landscape exploration for swarms.",
        "triggers": ["kennedy", "eberhart", "particle-swarm", "pso-optimization", "swarm-intelligence", "fitness-landscape", "velocity-update"],
        "foundations": [
            "PSO Velocity Update: v_i(t+1) = w*v_i(t) + c_1*r_1*(pbest_i - x_i(t)) + c_2*r_2*(gbest - x_i(t)).",
            "Inertia Weight w: Balances exploration (high w) vs exploitation (low w), typically decayed dynamically over iterations.",
            "Socio-Cognitive Duality: Individual cognitive memory (pbest) balances social consensus (gbest) to escape local optima in solution space.",
        ],
        "protocol": "Optimize agent hyper-parameters, prompt candidates, or architecture topology using Particle Swarm dynamics. Balance autonomous exploration with flock-wide best solution exploitation.",
        "anti_patterns": [
            "Premature convergence to suboptimal local minima due to excessive social attraction weight.",
            "Static velocity parameters causing particles to oscillate wildly across fitness boundaries.",
        ],
    },
    {
        "id": "agentic-bonabeau-swarm-stigmergy",
        "book": "Swarm Intelligence: From Natural to Artificial Systems - Eric Bonabeau, Marco Dorigo & Guy Theraulaz",
        "desc": "Stigmergic coordination, indirect communication via digital traces, division of labor, threshold response models, and collective ant trail algorithms.",
        "triggers": ["bonabeau", "dorigo", "theraulaz", "stigmergy", "digital-pheromones", "ant-algorithms", "division-of-labor", "threshold-response"],
        "foundations": [
            "Sematectonic Stigmergy: Environmental modification itself directs future actions (e.g. code artifacts and test results guide subsequent agent steps).",
            "Sign-based Stigmergy: Specialized signals (digital pheromones, cache tags, event logs) guide collective agent routing.",
            "Response Threshold Model: Agent i undertakes task j when stimulus s_j exceeds internal threshold theta_ij: P(perform) = s_j^2 / (s_j^2 + theta_ij^2).",
        ],
        "protocol": "Coordinate large agent swarms via stigmergic environmental traces rather than point-to-point RPCs. Let repository state, error logs, and build artifacts act as pheromones driving agent task pickup.",
        "anti_patterns": [
            "Flooding agents with synchronous point-to-point messages instead of reading environmental cues.",
            "Pheromone buildup without evaporation, leading to stale historical artifacts dominating decisions.",
        ],
    },
    {
        "id": "agentic-camazine-self-organization",
        "book": "Self-Organization in Biological Systems - Scott Camazine et al.",
        "desc": "Positive and negative feedback loops, symmetry breaking, quorum sensing, and spontaneous pattern formation in biological and software multi-agent swarms.",
        "triggers": ["camazine", "self-organization", "quorum-sensing", "symmetry-breaking", "feedback-loops", "spontaneous-order"],
        "foundations": [
            "Feedback Balance: Positive feedback amplifies micro-perturbations (innovation); negative feedback enforces stability and resource bounds.",
            "Quorum Sensing: Collective transitions trigger only when local interaction density exceeds a critical concentration threshold.",
            "Symmetry Breaking: Homogeneous agents differentiate into specialized functional roles through stochastic fluctuations and local reinforcement.",
        ],
        "protocol": "Design resilient self-organizing agent networks where global order emerges from simple local rules. Implement quorum sensing for distributed consensus before committing major refactors.",
        "anti_patterns": [
            "Unchecked positive feedback causing runaway compute loops or cascading agent hallucinations.",
            "Rigid top-down assignment of roles that cannot adapt to dynamic workload fluctuations.",
        ],
    },
    {
        "id": "agentic-reynolds-autonomous-agents",
        "book": "Flocks, Herds, and Schools: A Distributed Behavioral Model - Craig Reynolds",
        "desc": "Autonomous steering behaviors, separation, alignment, cohesion, obstacle avoidance, and decentralized crowd dynamics for autonomous software agents.",
        "triggers": ["reynolds", "boids", "steering-behaviors", "separation-alignment-cohesion", "autonomous-flocking", "flocking-agents"],
        "foundations": [
            "Separation: Steer to avoid crowding local flockmates: F_sep = sum((x_i - x_j) / ||x_i - x_j||^2).",
            "Alignment: Steer towards the average heading of local flockmates: F_align = mean(v_j) - v_i.",
            "Cohesion: Steer to move toward the average position (center of mass) of local flockmates: F_coh = mean(x_j) - x_i.",
        ],
        "protocol": "Use Reynolds flocking forces to coordinate concurrent code generation tasks. Keep agents separated in code scopes, aligned on architectural standards, and cohesive around the project vision.",
        "anti_patterns": [
            "Overlapping code edits causing git merge conflicts (separation failure).",
            "Incompatible architectural decisions across modules (alignment failure).",
        ],
    },
    {
        "id": "agentic-dorigo-ant-colony",
        "book": "Ant Colony Optimization - Marco Dorigo & Thomas Stützle",
        "desc": "Pheromone deposition, evaporation dynamics, artificial ant graph routing, combinatorial optimization, and heuristic search across solution spaces.",
        "triggers": ["dorigo", "ant-colony", "aco-optimization", "pheromone-evaporation", "graph-routing", "combinatorial-search"],
        "foundations": [
            "Pheromone Evaporation: tau_ij(t+1) = (1 - rho) * tau_ij(t) + sum(Delta_tau_ij^k), where rho in (0, 1] is evaporation rate.",
            "Probabilistic Transition Rule: p_ij^k = [tau_ij]^alpha * [eta_ij]^beta / sum([tau_il]^alpha * [eta_il]^beta), trading off trail history vs local heuristic.",
            "Pheromone Reinforcement: Successful solution paths deposit reinforcement inversely proportional to path length or compilation cost.",
        ],
        "protocol": "Route agent problem solving through complex call graphs and dependency trees using Ant Colony Optimization. Evaporate failed solution attempts and reinforce verified green test paths.",
        "anti_patterns": [
            "Zero evaporation rate causing premature lock-in on suboptimal legacy code paths.",
            "Ignoring local heuristic eta_ij (e.g. type checking error count), relying blindly on pheromones.",
        ],
    },
    {
        "id": "agentic-murray-consensus-cooperation",
        "book": "Consensus and Cooperation in Networked Multi-Agent Systems - Reza Olfati-Saber, J. Alex Fax & Richard M. Murray",
        "desc": "Graph Laplacians, algebraic connectivity, consensus protocols, agreement algorithms under directed delay topologies, and distributed swarm formation control.",
        "triggers": ["murray", "olfati-saber", "consensus-cooperation", "graph-laplacian", "algebraic-connectivity", "agreement-protocol", "swarm-consensus"],
        "foundations": [
            "Continuous-time Consensus: dx_i/dt = - sum_j a_ij * (x_i - x_j) = - [L * x]_i, where L = D - A is the graph Laplacian matrix.",
            "Algebraic Connectivity lambda_2(L): Consensus is achieved asymptotically if and only if the communication network graph has a spanning tree (lambda_2 > 0).",
            "Delay Robustness: Nyquist criteria bounds the maximum allowable communication delay tau < pi / (2 * lambda_max(L)) for stability.",
        ],
        "protocol": "Enforce distributed state consensus across agent swarms using Laplacian feedback. Verify algebraic connectivity of the communication graph to guarantee synchronization before execution.",
        "anti_patterns": [
            "Partitioned communication graphs where disconnected agent clusters diverge into incompatible states.",
            "Network message latency exceeding theoretical stability bounds, triggering oscillation.",
        ],
    },

    # =========================================================================
    # Cluster 2: Foundation Models, In-Context Learning & Prompt Architecture (11-20)
    # =========================================================================
    {
        "id": "agentic-alammar-transformer-mechanics",
        "book": "Hands-On Large Language Models / The Illustrated Transformer - Jay Alammar & Maarten Grootendorst",
        "desc": "Multi-head self-attention mechanics, KV-cache management, rotary positional embeddings (RoPE), token logits, and internal transformer layer projections.",
        "triggers": ["alammar", "illustrated-transformer", "transformer-mechanics", "multi-head-attention", "kv-cache", "rope-embeddings", "token-logits"],
        "foundations": [
            "Scaled Dot-Product Attention: Attention(Q, K, V) = softmax(Q * K^T / sqrt(d_k)) * V.",
            "KV-Cache Memory Footprint: Memory = 2 * n_layers * n_heads * d_head * seq_len * batch_size * precision_bytes.",
            "Rotary Positional Embeddings (RoPE): Embeds positional information through complex rotation matrices preserving relative token distances.",
        ],
        "protocol": "Optimize agent prompts for KV-cache reuse by placing static system prompts and tools at the absolute beginning of context. Manage token budget strictly relative to context window limits.",
        "anti_patterns": [
            "Dynamic prefix mutation that breaks KV-cache prefix sharing across turns, multiplying inference latency.",
            "Exceeding attention budget leading to needle-in-a-haystack retrieval degradation.",
        ],
    },
    {
        "id": "agentic-huyen-ai-engineering",
        "book": "AI Engineering: Building Applications with Foundation Models - Chip Huyen",
        "desc": "Production AI systems, prompt chaining, structured outputs (JSON schema), latency/cost trade-offs, evaluation cascades, and enterprise agent deployment.",
        "triggers": ["huyen", "chip-huyen", "ai-engineering", "structured-outputs", "evaluation-cascades", "prompt-chaining", "foundation-models"],
        "foundations": [
            "Structured Output Enforcement: Guarantee valid JSON/Pydantic schemas via grammar-constrained sampling or JSON mode.",
            "Cost-Latency-Quality Frontier: Route queries across small local models (fast/cheap) and frontier cloud models based on task complexity.",
            "Evaluation Cascades: Multi-stage evaluation combining regex unit tests, AST parsers, and LLM-as-a-judge rubrics.",
        ],
        "protocol": "Implement strict JSON schema contracts for all agent tool calls. Route routine code tasks to local LLMs and escalate architectural refactoring to frontier models.",
        "anti_patterns": [
            "Allowing unconstrained free-text output when deterministic JSON schemas are required by downstream tools.",
            "Using expensive frontier models for trivial regex extractions or boilerplate formatting.",
        ],
    },
    {
        "id": "agentic-rothman-transformers-nlp",
        "book": "Transformers for Natural Language Processing and Computer Vision - Denis Rothman",
        "desc": "Transformer tokenization, attention visualization, encoder-decoder vs decoder-only routing, few-shot conditioning, and downstream agent specialization.",
        "triggers": ["rothman", "transformers-nlp", "attention-visualization", "encoder-decoder", "decoder-only", "few-shot-conditioning"],
        "foundations": [
            "Subword Tokenization (BPE/WordPiece): Text tokenization maps vocabulary IDs preserving morphological subword units.",
            "Attention Map Analysis: Inspecting cross-attention weights to audit whether the agent attends to relevant codebase context.",
            "Few-Shot Exemplar Anchoring: Injecting input-output exemplars with explicit reasoning traces to constrain output variance.",
        ],
        "protocol": "Provide 2-3 precise input/output exemplars within agent instructions to anchor tone, formatting, and structural invariants. Verify tokenization boundaries on specialized code syntax.",
        "anti_patterns": [
            "Assuming word-level tokenization for code identifiers, causing token fragmentation in camelCase/snake_case.",
            "Providing conflicting few-shot examples that induce high epistemic entropy.",
        ],
    },
    {
        "id": "agentic-tunstall-nlp-transformers",
        "book": "Natural Language Processing with Transformers - Lewis Tunstall, Leandro von Werra & Thomas Wolf",
        "desc": "Parameter-efficient fine-tuning (PEFT/LoRA), quantization (int8/int4), model deployment pipelines, instruction tuning, and local LLM execution.",
        "triggers": ["tunstall", "von-werra", "wolf", "huggingface", "peft-lora", "quantization", "instruction-tuning", "local-llm"],
        "foundations": [
            "Low-Rank Adaptation (LoRA): W_updated = W_0 + (alpha / r) * B * A, where rank r << min(d_in, d_out).",
            "Quantization Trade-offs: Quantizing weights to 4-bit (AWQ/GPTQ) reduces VRAM by ~70% with negligible perplexity penalty on code tasks.",
            "Instruction Dataset Curation: Clean, deduplicated, verified test-passing code samples maximize transfer learning during alignment.",
        ],
        "protocol": "Deploy quantized local models for offline agent coding loops. Use LoRA adapters tailored to specific internal frameworks and domain-specific APIs.",
        "anti_patterns": [
            "Deploying unquantized 32-bit float models for CLI agents, causing GPU out-of-memory crashes.",
            "Fine-tuning on unverified code containing syntax errors and security vulnerabilities.",
        ],
    },
    {
        "id": "agentic-jurafsky-slp-language-models",
        "book": "Speech and Language Processing (3rd ed) - Daniel Jurafsky & James H. Martin",
        "desc": "Autoregressive language modeling, perplexity, beam search decoding, temperature and top-p sampling, semantic parsing, and linguistic foundations.",
        "triggers": ["jurafsky", "martin", "speech-language-processing", "beam-search", "nucleus-sampling", "semantic-parsing", "perplexity"],
        "foundations": [
            "Autoregressive Probability Chain: P(w_1, ..., w_n) = prod_{i=1}^n P(w_i | w_1, ..., w_{i-1}).",
            "Nucleus (Top-p) Sampling: Restricts generation candidate set to smallest subset V^(p) where sum_{w in V^(p)} P(w) >= p.",
            "Perplexity (PPL): PPL(W) = exp(- 1/N * sum_{i=1}^N ln P(w_i | w_{<i})), measuring model uncertainty.",
        ],
        "protocol": "Set temperature to 0.0 for deterministic code generation and property verification. Tune top-p to 0.95 for exploratory architecture brainstorming.",
        "anti_patterns": [
            "High temperature (> 0.7) during syntax-critical refactoring, inducing hallucinated import statements.",
            "Using greedy search when multi-candidate beam search is required for complex constraint satisfaction.",
        ],
    },
    {
        "id": "agentic-goldberg-nn-nlp",
        "book": "Neural Network Methods for Natural Language Processing - Yoav Goldberg",
        "desc": "Continuous vector representations, dense embeddings, compositional semantics, feedforward and recurrent networks, and geometric embedding spaces.",
        "triggers": ["goldberg", "neural-nlp", "dense-embeddings", "vector-representations", "compositional-semantics", "embedding-geometry"],
        "foundations": [
            "Distributional Hypothesis: Words occurring in similar contexts share similar semantic representations in vector space.",
            "Vector Cosine Similarity: sim(u, v) = (u . v) / (||u|| * ||v||), invariant to vector magnitude.",
            "Compositionality: Representing complex phrases or code blocks as geometric compositions of their atomic token vectors.",
        ],
        "protocol": "Index codebases in high-dimensional vector spaces using semantic embeddings. Compute cosine similarity against user problem statements to retrieve relevant source files.",
        "anti_patterns": [
            "Relying solely on vector embeddings for exact identifier search where inverted lexical search is required.",
            "Comparing embeddings across heterogeneous vector spaces without shared alignment.",
        ],
    },
    {
        "id": "agentic-ravichandiran-llm-engineering",
        "book": "Getting Started with Large Language Models - Sudharsan Ravichandiran",
        "desc": "LangChain & LlamaIndex internals, agent chains, tool use integrations, prompt templates, memory buffers, and orchestration pipelines.",
        "triggers": ["ravichandiran", "llm-chains", "tool-integration", "prompt-templates", "agent-orchestration", "memory-buffers"],
        "foundations": [
            "Tool Calling Protocol: The LLM emits structured tool invocation tokens which the runtime executes and returns as tool result messages.",
            "Conversational Buffer Memory: Rolling memory buffers prune historical messages to prevent context exhaustion while retaining core directives.",
            "Chaining Paradigm: Composable pipeline execution where output of step N feeds input of step N+1 under invariant assertions.",
        ],
        "protocol": "Construct agent workflows as deterministic Directed Acyclic Graphs (DAGs). Isolate tool execution in sandboxed environments with strict timeouts and error handling.",
        "anti_patterns": [
            "Unbounded memory buffers growing until the context window overflows and triggers runtime crashes.",
            "Allowing tools to execute destructive shell commands without human-in-the-loop verification.",
        ],
    },
    {
        "id": "agentic-briggs-prompt-engineering",
        "book": "Prompt Engineering for Generative AI - James Briggs & Francisco Ingham",
        "desc": "Chain-of-Thought (CoT), Tree-of-Thoughts (ToT), directional stimulus prompting, few-shot exemplars, prompt injection defenses, and system prompt hardening.",
        "triggers": ["briggs", "ingham", "prompt-engineering", "chain-of-thought", "tree-of-thoughts", "prompt-injection-defense", "system-prompt-hardening"],
        "foundations": [
            "Chain-of-Thought Reasoning: Eliciting intermediate reasoning steps significantly boosts performance on multi-step algorithmic deduction.",
            "Tree-of-Thoughts (ToT): Exploration of branching thought trajectories evaluated by self-assessment scoring and backtracking.",
            "Prompt Injection Boundary Delimiters: Enclosing user inputs in strict XML/Markdown fences (<user_input>...</user_input>) to prevent instruction hijacking.",
        ],
        "protocol": "Enforce explicit 'Thinking' blocks before code synthesis. Sanitize and isolate all untrusted inputs with boundary tags and anti-injection instructions.",
        "anti_patterns": [
            "Permitting raw user text to concatenate directly with system instructions without sanitization delimiters.",
            "Skipping scratchpad reasoning on non-trivial algorithmic tasks, leading to logic flaws.",
        ],
    },
    {
        "id": "agentic-kamphuis-in-context-reasoning",
        "book": "The Art of Asking AI - Nathan Kamphuis",
        "desc": "In-context scaffolding, role and persona definition, iterative conversational steering, constraint anchoring, and cognitive scaffolding for coding agents.",
        "triggers": ["kamphuis", "in-context-reasoning", "cognitive-scaffolding", "persona-definition", "conversational-steering", "constraint-anchoring"],
        "foundations": [
            "Persona Anchoring: Priming the agent as an elite principal systems engineer focuses the conditional probability distribution toward robust code patterns.",
            "Negative Constraint Priming: Explicitly enumerating forbidden patterns ('DO NOT use deprecated API X') reduces error rates.",
            "Iterative Narrowing: Guiding the agent from high-level architectural specification down to function-level implementation.",
        ],
        "protocol": "Anchor the agent's persona with precise domain expertise. Clearly state architectural invariants, performance targets, and forbidden dependencies upfront.",
        "anti_patterns": [
            "Vague, generic prompts ('write code for X') that yield superficial or incomplete implementations.",
            "Stating what to do without stating what NOT to do, allowing anti-patterns to seep in.",
        ],
    },
    {
        "id": "agentic-chollet-deep-learning-intuition",
        "book": "Deep Learning with Python - François Chollet",
        "desc": "Representation learning, geometric transformations, generalization vs memorization, gradient descent intuition, and foundational deep learning mechanics.",
        "triggers": ["chollet", "representation-learning", "generalization-memorization", "geometric-transformations", "deep-learning-intuition", "manifold-hypothesis"],
        "foundations": [
            "Manifold Hypothesis: Real-world high-dimensional data concentrates near low-dimensional non-linear manifolds in embedding space.",
            "Generalization vs Memorization: True intelligence is the ability to adapt to new situations using compact abstraction models rather than memorizing training data.",
            "Differentiable Programming: Composing differentiable computational graphs optimized via chain-rule backpropagation.",
        ],
        "protocol": "Design agent reasoning architectures that build generalized mental models of codebases rather than memorizing brittle surface strings. Verify zero-shot edge cases.",
        "anti_patterns": [
            "Overfitting agent prompts to narrow test inputs, failing on unseen user scenarios.",
            "Treating neural models as infallible databases rather than probabilistic statistical representations.",
        ],
    },

    # =========================================================================
    # Cluster 3: Autonomous Planning, Reasoning & Decision Engines (21-30)
    # =========================================================================
    {
        "id": "agentic-russell-norvig-aima",
        "book": "Artificial Intelligence: A Modern Approach (4th ed) - Stuart Russell & Peter Norvig",
        "desc": "Rational agent framework, PEAS (Performance, Environment, Actuators, Sensors), utility theory, adversarial search, Markov decision processes, and knowledge representation.",
        "triggers": ["russell-norvig", "aima", "rational-agents", "peas-framework", "utility-theory", "adversarial-search", "markov-decision-process"],
        "foundations": [
            "PEAS Formalization: Define agent's Performance measure, Environment properties (deterministic vs stochastic, fully vs partially observable), Actuators, and Sensors.",
            "Principle of Maximum Expected Utility (MEU): A rational agent chooses action a* = argmax_a sum_s' P(s' | s, a) * U(s').",
            "State-Space Search Graph: Representing problem states as nodes and transitions as edges traversed via admissibility-guaranteed algorithms.",
        ],
        "protocol": "Define explicit PEAS boundaries for every software engineering agent. Calculate expected utility over potential refactor strategies before modifying core files.",
        "anti_patterns": [
            "Building agents without defining measurable performance metrics or success predicates.",
            "Assuming a fully observable deterministic environment when dealing with distributed networks or asynchronous compilers.",
        ],
    },
    {
        "id": "agentic-ghallab-automated-planning",
        "book": "Automated Planning: Theory and Practice - Malik Ghallab, Dana Nau & Paolo Traverso",
        "desc": "STRIPS and PDDL state representations, preconditions/effects, forward/backward state-space search, and Hierarchical Task Networks (HTN) for multi-step engineering.",
        "triggers": ["ghallab", "automated-planning", "strips", "pddl", "htn-planning", "preconditions-effects", "state-space-search"],
        "foundations": [
            "STRIPS Action Representation: Action a = (pre(a), add(a), del(a)), updating world state S' = (S \\ del(a)) union add(a).",
            "Hierarchical Task Networks (HTN): Decomposing high-level abstract tasks into partially ordered networks of primitive executable actions.",
            "Plan Soundness & Completeness: A plan is sound if every action precondition is satisfied and the terminal state satisfies the goal condition.",
        ],
        "protocol": "Formalize complex multi-step coding plans as HTN task trees. Guard every tool invocation with explicit precondition checks (e.g. file exists, git branch clean).",
        "anti_patterns": [
            "Executing destructive write actions without verifying preconditions, causing irrecoverable workspace corruption.",
            "Flat unorganized task lists that fail to model dependencies between compilation, testing, and deployment.",
        ],
    },
    {
        "id": "agentic-geffner-heuristic-search-planning",
        "book": "A Concise Introduction to Models and Methods for Automated Planning - Hector Geffner & Blai Bonet",
        "desc": "Delete-relaxation heuristics, landmark heuristics, state-space reduction, and satisficing vs optimal search in complex task graphs.",
        "triggers": ["geffner", "bonet", "heuristic-search-planning", "delete-relaxation", "landmark-heuristics", "satisficing-planning"],
        "foundations": [
            "Delete-Relaxation Heuristic (h+): Ignoring negative effects of actions yields an admissible relaxation of the true distance to the goal.",
            "Fact & Action Landmarks: Subgoals that must be true at some point in every valid plan, providing mandatory stepping stones.",
            "Satisficing Search: Trading bounded suboptimality for polynomial-time planning speed using Enforced Hill-Climbing or Greedy Best-First Search.",
        ],
        "protocol": "Identify architectural landmarks (e.g. database schema migrated, interface trait compiled) before writing implementation code. Use satisficing heuristics for rapid iteration.",
        "anti_patterns": [
            "Exhaustive brute-force search over massive solution spaces when satisficing greedy search suffices.",
            "Abandoning landmark milestones, leading to circular refactoring with no measurable progress.",
        ],
    },
    {
        "id": "agentic-sutton-barto-reinforcement-learning",
        "book": "Reinforcement Learning: An Introduction (2nd ed) - Richard S. Sutton & Andrew G. Barto",
        "desc": "Bellman equations, Markov Decision Processes (MDP), temporal-difference learning, policy gradients, exploration-exploitation trade-off, and credit assignment.",
        "triggers": ["sutton-barto", "reinforcement-learning", "bellman-equation", "markov-decision-process", "temporal-difference", "policy-gradient", "exploration-exploitation"],
        "foundations": [
            "Bellman Optimality Equation: V*(s) = max_a sum_{s', r} p(s', r | s, a) * [r + gamma * V*(s')].",
            "Temporal-Difference Error: delta_t = R_{t+1} + gamma * V(S_{t+1}) - V(S_t), enabling learning without complete episode rollout.",
            "Epsilon-Greedy Exploration: Balance exploiting known green test paths with epsilon probability of exploring novel architectural approaches.",
        ],
        "protocol": "Score agent code modifications using explicit reward functions (compiler clean = +10, test pass = +50, regression = -100). Apply credit assignment to isolate failing commits.",
        "anti_patterns": [
            "Setting discount factor gamma too low, causing myopic fixes that break future extensibility.",
            "Reward hacking where an agent comments out tests to falsely achieve a 100% pass rate.",
        ],
    },
    {
        "id": "agentic-pearl-heuristic-search",
        "book": "Heuristics: Intelligent Search Strategies for Computer Problem Solving - Judea Pearl",
        "desc": "A* search admissibility, monotone consistency, branch-and-bound, heuristic pruning, minimax game trees, and computational complexity of heuristics.",
        "triggers": ["pearl", "heuristic-search", "a-star-algorithm", "admissible-heuristic", "monotone-consistency", "branch-and-bound"],
        "foundations": [
            "A* Algorithm Evaluation: f(n) = g(n) + h(n), where g(n) is exact cost from start to n, and h(n) is estimated cost to goal.",
            "Admissibility & Optimality: If h(n) <= h*(n) (never overestimates true cost), A* is guaranteed to find the optimal path without expanding redundant nodes.",
            "Consistency (Monotonicity): h(n) <= c(n, a, n') + h(n'), guaranteeing that f-scores along any path never decrease.",
        ],
        "protocol": "Implement code refactor planners using A* search over AST modification graphs. Ensure the distance heuristic to passing test suites is strictly admissible.",
        "anti_patterns": [
            "Inadmissible heuristics that prune the true optimal code fix in favor of shallow hacks.",
            "Ignoring duplicate state detection, resulting in infinite loops in cyclic code graphs.",
        ],
    },
    {
        "id": "agentic-kaelbling-pomdp-planning",
        "book": "Planning and Acting in Partially Observable Stochastic Domains (POMDPs) - Leslie Pack Kaelbling, Michael L. Littman & Anthony R. Cassandra",
        "desc": "Belief state updates, observation uncertainty, policy trees, value iteration in continuous probability simplex, and active sensing in partially observable environments.",
        "triggers": ["kaelbling", "littman", "pomdp", "partially-observable", "belief-state", "observation-probability", "active-sensing"],
        "foundations": [
            "Belief State Update: b'(s') = P(o | s', a) * sum_s P(s' | s, a) * b(s) / P(o | b, a).",
            "POMDP Tuple: Defined as (S, A, T, R, Omega, O, gamma), where Omega is observation space and O is observation probability.",
            "Active Information Gathering: Selecting actions whose primary utility is reducing epistemic entropy over system state (e.g. running diagnostics).",
        ],
        "protocol": "Model unseen third-party API states and legacy systems as POMDPs. Execute active probe queries (diagnostics, logs, dry-runs) to collapse belief state uncertainty before mutating code.",
        "anti_patterns": [
            "Assuming complete observability of production systems, leading to blind overwrites of hidden state.",
            "Ignoring observation noise (flaky tests) and misclassifying system health.",
        ],
    },
    {
        "id": "agentic-thrun-probabilistic-robotics",
        "book": "Probabilistic Robotics - Sebastian Thrun, Wolfram Burgard & Dieter Fox",
        "desc": "Recursive Bayesian state estimation, Kalman filters, particle filters, SLAM (Simultaneous Localization and Mapping), and sensor fusion for autonomous agents.",
        "triggers": ["thrun", "burgard", "fox", "probabilistic-robotics", "bayesian-estimation", "kalman-filter", "particle-filter", "slam"],
        "foundations": [
            "Bayes Filter Loop: Prediction: bel_bar(x_t) = int p(x_t | u_t, x_{t-1}) * bel(x_{t-1}) dx_{t-1}; Correction: bel(x_t) = eta * p(z_t | x_t) * bel_bar(x_t).",
            "Particle Filtering: Representing arbitrary multimodal belief distributions via sets of weighted hypotheses (particles).",
            "Sensor Fusion: Combining telemetry from unit tests, linter outputs, and runtime metrics to estimate system reliability.",
        ],
        "protocol": "Maintain a Bayesian belief distribution over possible root causes during debugging. Update probabilities as compiler errors and diagnostic logs arrive.",
        "anti_patterns": [
            "Single-hypothesis fixation during debugging, ignoring contradictory diagnostic signals.",
            "Treating noisy runtime metrics as absolute ground truth without Bayesian filtering.",
        ],
    },
    {
        "id": "agentic-silver-mcts-decision-trees",
        "book": "Mastering the Game of Go without Human Knowledge / AlphaZero MCTS - David Silver et al.",
        "desc": "Monte Carlo Tree Search (MCTS), Upper Confidence Bound for Trees (UCT), selection-expansion-simulation-backpropagation loop, and self-play reasoning.",
        "triggers": ["silver", "alphazero", "mcts", "monte-carlo-tree-search", "uct-algorithm", "policy-value-networks", "self-play"],
        "foundations": [
            "UCT Formula: UCT(v) = Q(v) + c * sqrt(ln(N(parent)) / N(v)), balancing historical win-rate Q with exploration bonus.",
            "MCTS 4-Phase Loop: 1. Selection (traverse tree via UCT) -> 2. Expansion (add child node) -> 3. Simulation/Evaluation (rollout or value net) -> 4. Backpropagation (update visits and scores).",
            "Self-Play Alignment: Generating adversarial test cases against synthetic code implementations to discover edge-case regressions.",
        ],
        "protocol": "Explore alternative code refactoring branches using MCTS. Rank candidate implementations via automated test-run rollouts and select the branch with the highest cumulative reward.",
        "anti_patterns": [
            "Greedy depth-first exploration without backtracking, getting stuck in irrecoverable compilation traps.",
            "Zero exploration coefficient (c = 0), preventing the discovery of superior refactoring solutions.",
        ],
    },
    {
        "id": "agentic-yao-react-interleaved-reasoning",
        "book": "ReAct: Synergizing Reasoning and Acting in Language Models - Shunyu Yao et al.",
        "desc": "Interleaved Thought-Action-Observation loops, external knowledge grounding, hallucination interruption, and dynamic trajectory adjustment in language agents.",
        "triggers": ["yao", "react-framework", "thought-action-observation", "interleaved-reasoning", "tool-grounding", "hallucination-interruption"],
        "foundations": [
            "ReAct Triad: Thought_t -> Action_t -> Observation_t sequence where reasoning guides action and observation grounds reasoning.",
            "External Grounding Invariant: The agent MUST NOT hallucinate the results of tool calls; all state transitions depend strictly on Observation_t.",
            "Hallucination Interruption: If Observation_t contradicts Thought_t, the agent immediately enters a corrective Thought_{t+1} phase.",
        ],
        "protocol": "Execute all autonomous actions in a strict Thought -> Action -> Observation loop. Never combine multiple unverified actions into a single ungrounded assumption.",
        "anti_patterns": [
            "Emitting speculative observations instead of awaiting true tool return payloads.",
            "Skipping the Thought phase, devolving into unguided random tool thrashing.",
        ],
    },
    {
        "id": "agentic-shinn-reflexion-self-correction",
        "book": "Reflexion: Language Agents with Verbal Reinforcement Learning - Noah Shinn et al.",
        "desc": "Verbal memory reflection, self-evaluative scalar rewards, error retrospective analysis, trial-and-error retry loops, and episodic memory persistence.",
        "triggers": ["shinn", "reflexion", "verbal-reinforcement", "self-correction", "episodic-reflection", "error-retrospective"],
        "foundations": [
            "Verbal Memory Buffer: Storing structured retrospectives: 'Attempt failed because X. In next attempt, avoid Y and implement Z.'",
            "Self-Evaluative Heuristic: Evaluating final code artifacts against a scalar rubric (0-100) before presenting them to the user.",
            "Iterative Correction Loop: Persisting reflection logs across agent iterations to prevent repeating identical failure trajectories.",
        ],
        "protocol": "When a build or test suite fails, generate an explicit verbal reflection diagnosing the exact failure mechanism before attempting code edits. Store this reflection in episodic memory.",
        "anti_patterns": [
            "Repeatedly applying the same failing patch across multiple turns without verbal reflection.",
            "Discarding error retrospectives between attempts, losing hard-won debugging context.",
        ],
    },

    # =========================================================================
    # Cluster 4: Vibe Coding, Developer Flow, Ergonomics & Rapid Iteration (31-40)
    # =========================================================================
    {
        "id": "agentic-karpathy-vibe-coding-paradigm",
        "book": "Vibe Coding: The Paradigm of Intuitive Autonomous AI Software Creation - Andrej Karpathy",
        "desc": "Conversational code iteration, human-as-director, AI-as-synthesizer, high-velocity feedback loops, intuitive flow state, and prompt-first software engineering.",
        "triggers": ["karpathy", "vibe-coding", "conversational-programming", "human-as-director", "intuitive-iteration", "prompt-first-development"],
        "foundations": [
            "Prompt-Code Symbiosis: Natural language is the primary design syntax; generated code is an ephemeral compiler target verified by tests.",
            "Velocity Maximization: Minimize the feedback loop latency between human architectural intent and executable running artifacts.",
            "Director-Synthesizer Dualism: The human provides domain taste, aesthetic judgment, and safety boundaries; the AI agent synthesizes boilerplate and tests.",
        ],
        "protocol": "Enable rapid vibe coding loops. Let the developer specify high-level vibes and system boundaries; synthesize complete, fully functioning implementations with instant test validation.",
        "anti_patterns": [
            "Forcing the user to write tedious boilerplate syntax when intent is crystal clear.",
            "Interrupting the user's flow state with unnecessary pedantic confirmations on trivial details.",
        ],
    },
    {
        "id": "agentic-csikszentmihalyi-flow-state",
        "book": "Flow: The Psychology of Optimal Experience - Mihaly Csikszentmihalyi",
        "desc": "Challenge-skill equilibrium, clear proximal goals, unambiguous feedback, deep immersion, distortion of temporal perception, and cognitive ergonomics for developers.",
        "triggers": ["csikszentmihalyi", "flow-state", "challenge-skill-balance", "unambiguous-feedback", "cognitive-ergonomics", "developer-immersion"],
        "foundations": [
            "Flow Channel Condition: Task challenge C and operator skill S must remain balanced: C approx S. If C >> S -> anxiety; if S >> C -> boredom.",
            "Immediate Feedback Invariant: System responses must arrive within sub-second thresholds to prevent breaking the developer's working memory.",
            "Clear Proximal Subgoals: Deconstruct ambiguous epic goals into clear, incremental milestones achievable in minutes.",
        ],
        "protocol": "Preserve developer flow state at all costs. Provide immediate, deterministic feedback for every code edit and break daunting tasks into bite-sized achievable steps.",
        "anti_patterns": [
            "Unresponsive tools or long silent pauses without progress telemetry.",
            "Presenting overwhelming 50-step plans that induce cognitive overload and anxiety.",
        ],
    },
    {
        "id": "agentic-hunt-pragmatic-programmer",
        "book": "The Pragmatic Programmer: Your Journey to Mastery - David Thomas & Andrew Hunt",
        "desc": "Don't Repeat Yourself (DRY), orthogonality, tracer bullets, broken windows theory, stone soup, pragmatic paranoia, and engineering craftsmanship.",
        "triggers": ["hunt", "pragmatic-programmer", "dry-principle", "orthogonality", "tracer-bullets", "broken-windows", "pragmatic-paranoia"],
        "foundations": [
            "DRY Principle: Every piece of knowledge must have a single, unambiguous, authoritative representation within a system.",
            "Orthogonality: Eliminate side-effects between unrelated components so modifying module A cannot break module B.",
            "Tracer Bullets: Implement end-to-end thin vertical slices that connect all architectural layers before fleshing out bulk features.",
        ],
        "protocol": "Build vertical tracer bullets to validate end-to-end integration immediately. Never tolerate 'broken windows' (commented-out tests, unaddressed linter warnings).",
        "anti_patterns": [
            "Copy-pasting duplicate logic across multiple files, violating DRY.",
            "Building elaborate horizontal layers (data models, UI) without ever running end-to-end tracer tests.",
        ],
    },
    {
        "id": "agentic-raymond-cathedral-bazaar",
        "book": "The Cathedral and the Bazaar - Eric S. Raymond",
        "desc": "Release early and often, Linus's Law (many eyeballs make bugs shallow), treating users as co-developers, decentralized design, and open-source dynamics.",
        "triggers": ["raymond", "cathedral-bazaar", "release-early-often", "linus-law", "decentralized-development", "open-source-patterns"],
        "foundations": [
            "Linus's Law: Given enough eyeballs, all bugs are shallow (deploying automated testing and multi-agent review sweeps).",
            "Release Early, Release Often: Short release cadences minimize integration divergence and accelerate empirical feedback.",
            "Smart Data Structures: Smart data structures and dumb code work a lot better than the other way around.",
        ],
        "protocol": "Commit small, frequent, atomic changes that keep the build green. Use automated multi-agent code reviews to uncover hidden edge cases.",
        "anti_patterns": [
            "Massive multi-week PRs that are impossible to review or debug.",
            "Hoarding uncommitted changes locally, risking devastating merge conflicts.",
        ],
    },
    {
        "id": "agentic-graham-hackers-painters",
        "book": "Hackers & Painters: Big Ideas from the Computer Age - Paul Graham",
        "desc": "Software as creative craft, bottom-up design, sketch-driven prototyping, expressive language power, and rapid iterative hacking as thinking.",
        "triggers": ["graham", "hackers-painters", "software-craftsmanship", "bottom-up-design", "expressive-power", "rapid-prototyping"],
        "foundations": [
            "Sketching in Code: Software design is an empirical discovery process where coding directly reveals architectural possibilities.",
            "Bottom-Up Design: Build a layered domain language upward from primitives until solving the target problem becomes natural and concise.",
            "Succinctness is Power: High expressive density reduces cognitive surface area and the statistical probability of bugs.",
        ],
        "protocol": "Support bottom-up development by synthesizing clean domain primitives first. Enable exploratory prototyping that clarifies requirements through running software.",
        "anti_patterns": [
            "Overly bureaucratic top-down waterfall planning before writing any running code.",
            "Verbose, ceremonial boilerplate that obscures core business logic.",
        ],
    },
    {
        "id": "agentic-beck-extreme-programming",
        "book": "Extreme Programming Explained: Embrace Change - Kent Beck",
        "desc": "Pair programming, continuous integration, collective code ownership, small release increments, ruthless refactoring, and rapid user feedback loops.",
        "triggers": ["beck", "extreme-programming", "pair-programming", "continuous-integration", "collective-ownership", "ruthless-refactoring"],
        "foundations": [
            "Extreme Pair Programming: AI agent acts as the active navigator or driver in real-time pairing with the developer.",
            "Continuous Integration: Code is integrated into the trunk multiple times per day, validated by automated test suites.",
            "Ruthless Refactoring: Continuously simplify design, remove dead code, and improve readability without altering observable behavior.",
        ],
        "protocol": "Act as an indefatigable XP pair programmer. Suggest proactive refactorings, write missing regression tests, and maintain trunk health continuously.",
        "anti_patterns": [
            "Letting technical debt accumulate without refactoring.",
            "Treating code as private unchangeable property rather than collective shared assets.",
        ],
    },
    {
        "id": "agentic-ries-lean-mvp-feedback",
        "book": "The Lean Startup - Eric Ries",
        "desc": "Build-Measure-Learn feedback loops, Minimum Viable Product (MVP), pivot vs persevere, validated learning, and vanity vs actionable metrics.",
        "triggers": ["ries", "lean-startup", "build-measure-learn", "minimum-viable-product", "mvp", "validated-learning", "pivot-persevere"],
        "foundations": [
            "Build-Measure-Learn Cycle: The fundamental feedback loop of high-velocity engineering; minimize total time through this loop.",
            "Minimum Viable Product (MVP): The version of a new product which allows a team to collect the maximum amount of validated learning with the least effort.",
            "Actionable vs Vanity Metrics: Measure real system behavior (latency, conversion, test pass rate) rather than superficial vanity stats.",
        ],
        "protocol": "Help vibe coders ship MVPs rapidly to test core product hypotheses. Measure performance with actionable telemetry before investing in heavy infrastructure.",
        "anti_patterns": [
            "Premature scaling and over-engineering infrastructure for hypothetical future traffic.",
            "Building complex features without defining measurable validation criteria.",
        ],
    },
    {
        "id": "agentic-knapp-design-sprint-prototyping",
        "book": "Sprint: How to Solve Big Problems and Test New Ideas in Just Five Days - Jake Knapp, John Zeratsky & Braden Kowitz",
        "desc": "Timeboxed prototyping sprints, storyboarding, customer validation, facade prototypes, and rapid hypothesis testing without production code.",
        "triggers": ["knapp", "zeratsky", "design-sprint", "facade-prototyping", "storyboard-validation", "timeboxed-sprints", "rapid-hypothesis"],
        "foundations": [
            "Facade Prototyping: Build the illusion of a finished system (using mock APIs and synthetic fixtures) to validate UX and business value in hours.",
            "Timeboxing Discipline: Strict time limits force decision-making and prevent bikeshedding over non-critical edge cases.",
            "Storyboard Mapping: Map critical user journeys end-to-end before implementing backend plumbing.",
        ],
        "protocol": "Rapidly scaffold interactive UI mockups and realistic API stubs so developers can test real product workflows before writing complex backend databases.",
        "anti_patterns": [
            "Spending days configuring backend databases before validating whether anyone wants the feature.",
            "Endless open-ended meetings without timeboxed prototype deliverables.",
        ],
    },
    {
        "id": "agentic-norman-human-centered-interfaces",
        "book": "The Design of Everyday Things - Don Norman",
        "desc": "Affordances, signifiers, conceptual models, feedback visibility, error tolerance, and bridging the gulf of execution and evaluation.",
        "triggers": ["norman", "don-norman", "affordances-signifiers", "conceptual-models", "gulf-of-execution", "human-centered-design", "error-tolerance"],
        "foundations": [
            "Gulf of Execution & Evaluation: Execution: how easily can the user figure out what to do; Evaluation: how easily can the user interpret system state.",
            "Affordances & Signifiers: Affordances represent possible actions; signifiers communicate where and how action should take place.",
            "Forcing Functions: Design constraints that make it physically or logically impossible to make destructive mistakes.",
        ],
        "protocol": "Design developer CLI and UI experiences with intuitive affordances and explicit signifiers. Implement confirmation forcing functions for destructive actions.",
        "anti_patterns": [
            "Cryptic CLI tool flags with zero affordance or help feedback.",
            "Silent failures where operations complete with errors but return zero exit codes.",
        ],
    },
    {
        "id": "agentic-krug-intuitive-interaction",
        "book": "Don't Make Me Think - Steve Krug",
        "desc": "Cognitive load minimization, visual hierarchy, self-evident interfaces, mindless navigation, and ruthless omission of needless words in developer tools.",
        "triggers": ["krug", "dont-make-me-think", "cognitive-load-minimization", "visual-hierarchy", "self-evident-design", "frictionless-interaction"],
        "foundations": [
            "First Law of Usability: As far as humanly possible, interfaces should be self-evident and obvious without requiring a manual.",
            "Muddle vs Clarity: Clear visual hierarchy: things that are related visually belong together; primary actions dominate secondary actions.",
            "Omission of Needless Elements: Strip away boilerplate, extraneous text, and cognitive clutter from developer prompts and terminal outputs.",
        ],
        "protocol": "Format terminal and chat outputs with clean visual hierarchy, clear headings, and zero useless chatter. Make next steps completely obvious.",
        "anti_patterns": [
            "Dumping walls of unformatted markdown text that drown key warnings.",
            "Burying critical error remedies in verbose paragraph explanations.",
        ],
    },

    # =========================================================================
    # Cluster 5: Memory Systems, Knowledge Retrieval, RAG & Vector Space (41-50)
    # =========================================================================
    {
        "id": "agentic-lewis-rag-foundations",
        "book": "Retrieval-Augmented Generation for Knowledge-Intensive NLP Tasks - Patrick Lewis et al.",
        "desc": "Dense passage retrieval, parametric vs non-parametric memory, cross-entropy loss over retrieved docs, and hybrid generation architecture.",
        "triggers": ["lewis", "rag-foundations", "retrieval-augmented-generation", "dense-passage-retrieval", "non-parametric-memory", "hybrid-retrieval"],
        "foundations": [
            "RAG Generation Probability: P(y | x) = sum_{z in top-k} P(z | x) * P(y | x, z), marginalizing over retrieved document passages z.",
            "Dual-Memory Paradigm: Parametric memory (frozen LLM neural weights) augmented with non-parametric memory (vector database of source documents).",
            "Context Chunk Size Optimization: Finding the balance between semantic completeness (large chunks) and embedding retrieval precision (small chunks).",
        ],
        "protocol": "Augment agent generation with dense vector retrieval over codebase docs and history. Always cite specific file and line-number references for retrieved context.",
        "anti_patterns": [
            "Relying purely on parametric memory for internal codebase APIs, generating hallucinated methods.",
            "Injecting massive irrelevantly retrieved chunks that pollute context and cause hallucination.",
        ],
    },
    {
        "id": "agentic-manning-information-retrieval",
        "book": "Introduction to Information Retrieval - Christopher D. Manning, Prabhakar Raghavan & Hinrich Schütze",
        "desc": "Inverted indices, BM25 scoring, TF-IDF vector space model, cosine similarity, precision/recall curves, and text normalization.",
        "triggers": ["manning", "raghavan", "schutze", "information-retrieval", "inverted-index", "bm25-scoring", "tf-idf", "precision-recall"],
        "foundations": [
            "BM25 Scoring Formula: score(D, Q) = sum_{i=1}^n IDF(q_i) * (f(q_i, D) * (k_1 + 1)) / (f(q_i, D) + k_1 * (1 - b + b * (|D| / avgdl))).",
            "Inverted Index Invariant: O(1) post-list lookup mapping term tokens directly to document frequency and document IDs.",
            "Precision vs Recall Trade-off: Precision = TP / (TP + FP); Recall = TP / (TP + FN); optimize F1 score for code search.",
        ],
        "protocol": "Use hybrid search combining BM25 keyword matching (for exact variable/function names) with vector search (for conceptual semantic queries).",
        "anti_patterns": [
            "Using vector similarity alone to find exact identifier definitions (e.g. `UserAuthenticationHandler`).",
            "Failing to normalize code tokens (casing, camelCase splitting), leading to index misses.",
        ],
    },
    {
        "id": "agentic-baeza-yates-vector-retrieval",
        "book": "Modern Information Retrieval - Ricardo Baeza-Yates & Berthier Ribeiro-Neto",
        "desc": "Vector ranking models, probabilistic retrieval, index compression, evaluation metrics (MAP, NDCG), and query expansion algorithms.",
        "triggers": ["baeza-yates", "ribeiro-neto", "modern-retrieval", "ndcg-ranking", "query-expansion", "vector-ranking", "probabilistic-retrieval"],
        "foundations": [
            "Normalized Discounted Cumulative Gain (NDCG): NDCG_p = DCG_p / IDCG_p, where DCG_p = sum_{i=1}^p (2^{rel_i} - 1) / log_2(i + 1).",
            "Query Expansion: Augmenting short user queries with related synonyms, types, and compiler error signatures to improve retrieval recall.",
            "Rank Fusion (RRF): Reciprocal Rank Fusion: RRF(d) = sum_{m in models} 1 / (k + rank_m(d)), merging disparate retrieval rankings.",
        ],
        "protocol": "Implement Reciprocal Rank Fusion to combine lexical AST search, git blame logs, and vector embeddings. Score results using NDCG metrics.",
        "anti_patterns": [
            "Assuming top-1 retrieval is always accurate; always feed top-K diversified context chunks.",
            "Evaluating search pipelines without standard ground-truth relevance benchmarks.",
        ],
    },
    {
        "id": "agentic-malkov-hnsw-vector-indexing",
        "book": "Efficient and Robust Approximate Nearest Neighbor Search using HNSW Graphs - Yu. A. Malkov & D. A. Yashunin",
        "desc": "Hierarchical Navigable Small World (HNSW) graphs, skip-list topology, logarithmic search complexity, edge pruning heuristics, and vector index scaling.",
        "triggers": ["malkov", "yashunin", "hnsw-graphs", "approximate-nearest-neighbors", "ann-search", "vector-indexing", "skip-list-topology"],
        "foundations": [
            "Hierarchical Multilayer Graph: Multi-layer structure where top layers contain long-range skip edges and layer 0 contains dense local connectivity.",
            "Logarithmic Search Complexity: Search navigates greedy local minima across layers with average time complexity O(log N).",
            "Heuristic Edge Selection: Balances distance to candidates with angular diversity to prevent clustering and maintain navigability.",
        ],
        "protocol": "Configure HNSW index parameters (M, efConstruction, efSearch) for sub-10ms similarity queries across millions of code embeddings in local vector stores.",
        "anti_patterns": [
            "Using brute-force flat L2 search in production, causing unacceptable latency as codebase grows.",
            "Setting efSearch too low, degrading recall below acceptable thresholds for critical code retrieval.",
        ],
    },
    {
        "id": "agentic-anderson-actr-cognitive-memory",
        "book": "The Architecture of Cognition (ACT-R Memory) - John R. Anderson",
        "desc": "Declarative vs procedural memory, chunk activation equation, base-level learning, production rules, pattern matching, and cognitive memory retrieval.",
        "triggers": ["anderson", "act-r", "declarative-procedural-memory", "chunk-activation", "base-level-learning", "cognitive-architecture-memory"],
        "foundations": [
            "Base-Level Activation Equation: A_i = B_i + sum_j W_j * S_{ji} + epsilon, where B_i = ln(sum_{k=1}^n t_k^{-d}) decays as a power law of time.",
            "Declarative Chunks vs Production Rules: Facts/schemas reside in declarative memory; executable cognitive actions reside in procedural IF-THEN rules.",
            "Conflict Resolution: Selecting the production rule with the highest expected utility when multiple rules match the current goal buffer.",
        ],
        "protocol": "Model long-term agent memory using ACT-R activation equations. Decay historical conversation turns while boosting recently and frequently accessed code modules.",
        "anti_patterns": [
            "Treating all historical memories as equally relevant regardless of age or frequency of use.",
            "Mixing procedural execution logic with static declarative schemas.",
        ],
    },
    {
        "id": "agentic-baddeley-working-memory-buffers",
        "book": "Working Memory, Thought, and Action - Alan Baddeley",
        "desc": "Central executive, phonological loop, visuospatial sketchpad, episodic buffer, capacity limits in context windows, and cognitive load distribution.",
        "triggers": ["baddeley", "working-memory", "central-executive", "episodic-buffer", "cognitive-load", "context-window-management"],
        "foundations": [
            "Multi-Component Working Memory Model: Central Executive coordinates attention and controls three slave buffers: Phonological Loop, Visuospatial Sketchpad, and Episodic Buffer.",
            "Capacity Limits (Miller/Cowan): Working memory can reliably manipulate only 4-7 active conceptual chunks simultaneously.",
            "Episodic Buffer: Binds cross-modal information into coherent chronological episodes available for executive decision-making.",
        ],
        "protocol": "Structure LLM context windows to emulate Baddeley's working memory: an executive system prompt, a concise episodic buffer of recent turns, and scratchpad space.",
        "anti_patterns": [
            "Overloading the context window with dozens of unorganized code snippets exceeding the LLM's effective attention span.",
            "Failing to clear the working memory buffer after concluding an isolated subtask.",
        ],
    },
    {
        "id": "agentic-tulving-episodic-memory-retrieval",
        "book": "Elements of Episodic Memory - Endel Tulving",
        "desc": "Episodic vs semantic memory, autonoetic consciousness, retrieval cues, temporal tagging, chronesthesia, and agent trajectory retrospection.",
        "triggers": ["tulving", "episodic-memory", "semantic-memory", "autonoetic-consciousness", "temporal-tagging", "retrieval-cues"],
        "foundations": [
            "Episodic-Semantic Distinction: Semantic memory stores general knowledge ('Rust has ownership'); episodic memory stores temporally situated personal experiences ('In step 3 I broke the build').",
            "Encoding Specificity Principle: Retrieval is successful only if the cues present during retrieval match the information encoded with the memory trace.",
            "Chronesthesia: Mental time travel allowing an agent to simulate future outcomes by reconstructing past episodic trajectories.",
        ],
        "protocol": "Tag agent tool execution logs with precise temporal and situational metadata. Use encoding specificity to retrieve past debugging sessions that share identical error signatures.",
        "anti_patterns": [
            "Collapsing all historical actions into generic semantic rules, losing the chronological sequence of why decisions were made.",
            "Querying memory with generic keywords that lack the situational cues present during original failure.",
        ],
    },
    {
        "id": "agentic-sowa-knowledge-representation",
        "book": "Knowledge Representation: Logical, Philosophical, and Computational Foundations - John F. Sowa",
        "desc": "Conceptual graphs, first-order logic semantics, ontologies, semantic networks, semantic ambiguity resolution, and knowledge graph mapping.",
        "triggers": ["sowa", "knowledge-representation", "conceptual-graphs", "ontologies", "first-order-logic", "semantic-networks"],
        "foundations": [
            "Conceptual Graph Invariant: Bipartite graph of Concept nodes and Conceptual Relation nodes with formal first-order logic mappings.",
            "Ontological Commitment: Explicitly specifying the categories, relations, and invariants that exist within the software problem domain.",
            "Knowledge Fusion: Merging disparate semantic schemas via graph unification and constraint consistency checking.",
        ],
        "protocol": "Construct formal domain ontologies for target codebases. Model relationships (implements, extends, calls, imports) as typed conceptual graphs.",
        "anti_patterns": [
            "Unconstrained natural language summaries that introduce logical contradictions into the system's world model.",
            "Assuming isomorphic schemas across different microservices without explicit translation mappings.",
        ],
    },
    {
        "id": "agentic-baader-description-logics-ontologies",
        "book": "The Description Logic Handbook: Theory, Implementation, and Applications - Franz Baader et al.",
        "desc": "Description Logics (ALC, SHOIN), TBox (terminological) vs ABox (assertional) reasoning, tableau algorithms, and ontology subsumption.",
        "triggers": ["baader", "description-logics", "tbox-abox", "tableau-algorithm", "ontology-subsumption", "formal-knowledge-base"],
        "foundations": [
            "TBox vs ABox: TBox defines conceptual schema axioms (e.g. 'AdminUser subclass of User'); ABox defines concrete instance assertions (e.g. 'alice instance of AdminUser').",
            "Subsumption Checking: Determining if concept C is subsumed by concept D (C sqsubseteq D) under all valid interpretations.",
            "Tableau Decidability: Applying tableau expansion rules to systematically verify ontology satisfiability and consistency.",
        ],
        "protocol": "Verify that agent-generated architectural schemas and access control models are logically consistent using description logic subsumption checkers.",
        "anti_patterns": [
            "Defining cyclical ontology hierarchies with unsatisfiable concept definitions.",
            "Confusing class-level schema modifications (TBox) with instance-level data modifications (ABox).",
        ],
    },
    {
        "id": "agentic-robinson-graph-rag-knowledge",
        "book": "Graph Databases: New Opportunities for Connected Data - Ian Robinson, Jim Webber & Emil Eifrem",
        "desc": "Property graph models, graph traversal algorithms, Cypher queries, Knowledge Graph RAG, and multi-hop relationship reasoning across codebases.",
        "triggers": ["robinson", "webber", "eifrem", "graph-databases", "graph-rag", "property-graphs", "cypher-queries", "multi-hop-retrieval"],
        "foundations": [
            "Labeled Property Graph Model: Nodes with labels and key-value properties connected by directed, typed relationships with properties.",
            "Index-Free Adjacency: Each node directly references its adjacent neighbors, allowing O(1) traversal performance independent of total graph size.",
            "Multi-Hop Graph RAG: Traversing 2-3 degrees of separation (Function -> Calls -> Dependency -> Version) to retrieve complete architectural context.",
        ],
        "protocol": "Build a Graph RAG pipeline over codebase ASTs and dependency graphs. Use graph traversals to gather multi-hop context for complex refactorings.",
        "anti_patterns": [
            "Flat keyword search across disconnected files when understanding a bug requires walking call-graph paths.",
            "Unbounded breadth-first graph expansions that explode memory and retrieve irrelevant modules.",
        ],
    },

    # =========================================================================
    # Cluster 6: Code Generation, Program Synthesis, Compilers & DSLs (51-60)
    # =========================================================================
    {
        "id": "agentic-gulwani-program-synthesis",
        "book": "Program Synthesis - Sumit Gulwani, Oleksandr Polozov & Rishabh Singh",
        "desc": "Inductive program synthesis, programming by example (PBE), domain-specific Version Space Algebras, syntax-guided synthesis (SyGuS), and deductive search.",
        "triggers": ["gulwani", "polozov", "singh", "program-synthesis", "programming-by-example", "version-space-algebra", "sygus"],
        "foundations": [
            "Inductive Synthesis Invariant: Given input-output examples {(x_1, y_1), ..., (x_n, y_n)}, synthesize program P in DSL such that forall i, P(x_i) == y_i.",
            "Version Space Algebra (VSA): Compactly represent an exponential number of consistent candidate programs using a polynomial-sized shared DAG.",
            "Deductive Top-Down Search: Propagate input-output constraints downward through grammar operators to prune invalid program spaces early.",
        ],
        "protocol": "Synthesize data transformation pipelines and regex extractors using programming-by-example principles. Verify candidate programs against test suites before proposing them.",
        "anti_patterns": [
            "Proposing code without checking that it passes the user's provided input-output examples.",
            "Generating overly complex general programs when a simple DSL expression satisfies all constraints.",
        ],
    },
    {
        "id": "agentic-aho-dragon-compiler-parsing",
        "book": "Compilers: Principles, Techniques, and Tools (Dragon Book) - Alfred V. Aho, Monica S. Lam, Ravi Sethi & Jeffrey D. Ullman",
        "desc": "Lexical analysis, LL/LR parsing tables, abstract syntax trees (AST), syntax-directed translation, symbol tables, and compiler frontends.",
        "triggers": ["aho", "dragon-book", "compiler-parsing", "abstract-syntax-tree", "syntax-directed-translation", "symbol-table", "lr-parsing"],
        "foundations": [
            "Grammar Classification: Context-Free Grammar G = (V, Sigma, R, S) parsed via deterministic LR(1) or LALR tables without shift-reduce conflicts.",
            "AST Construction: Generating an Abstract Syntax Tree that abstracts away concrete punctuation while preserving hierarchical semantic structure.",
            "Symbol Table Scope Stack: Maintaining lexical scope hierarchies mapping identifier symbols to type signatures and memory offsets.",
        ],
        "protocol": "Parse agent-generated code into formal ASTs before saving to disk. Catch syntax and lexical errors immediately at the compiler frontend level.",
        "anti_patterns": [
            "Relying on naive regex string matching to inspect or refactor nested programming language constructs.",
            "Ignoring lexical scope rules, causing duplicate symbol declarations or shadow variable bugs.",
        ],
    },
    {
        "id": "agentic-cooper-compiler-ir-optimization",
        "book": "Engineering a Compiler - Keith D. Cooper & Linda Torczon",
        "desc": "Intermediate representations (IR), control flow graphs (CFG), SSA (Static Single Assignment) form, dead code elimination, and register allocation.",
        "triggers": ["cooper", "torczon", "compiler-optimization", "intermediate-representation", "control-flow-graph", "ssa-form", "dead-code-elimination"],
        "foundations": [
            "Static Single Assignment (SSA): Every variable is assigned exactly once; phi-nodes resolve values at confluence points in the Control Flow Graph.",
            "Dominator Tree Invariant: Node d dominates node n (d dom n) if every path from entry to n must pass through d.",
            "Dead Code Elimination: Iteratively removing operations whose definitions have no uses and produce no observable side-effects.",
        ],
        "protocol": "Analyze code refactorings at the Control Flow Graph and SSA level. Verify that transformations preserve dominance invariants and eliminate unreachable dead branches.",
        "anti_patterns": [
            "Refactorings that leave dangling unused variables, unreferenced imports, or unreachable code blocks.",
            "Accidentally altering phi-node value resolution across branching conditionals.",
        ],
    },
    {
        "id": "agentic-muchnick-cfg-dataflow-analysis",
        "book": "Advanced Compiler Design and Implementation - Steven S. Muchnick",
        "desc": "Dataflow equations (available expressions, reaching definitions, live variables), dominance frontiers, loop transformations, and interprocedural analysis.",
        "triggers": ["muchnick", "dataflow-analysis", "reaching-definitions", "live-variables", "dominance-frontiers", "interprocedural-analysis"],
        "foundations": [
            "Dataflow Equation Framework: Out[B] = Gen[B] union (In[B] \\ Kill[B]); In[B] = bigcup_{P in pred(B)} Out[P].",
            "Liveness Analysis: A variable is live at point p if there exists an execution path from p to a use that does not redefine the variable.",
            "Monotone Framework Fixed Point: Iterating dataflow equations until convergence is guaranteed by Knaster-Tarski fixed-point theorem on finite lattices.",
        ],
        "protocol": "Perform static dataflow analysis to ensure variables are initialized before use and resources (file handles, network sockets) are safely disposed along all paths.",
        "anti_patterns": [
            "Introducing uninitialized variable reads along rarely executed error branches.",
            "Resource leaks caused by failing to close handles on abnormal exit paths.",
        ],
    },
    {
        "id": "agentic-nystrom-crafting-interpreters",
        "book": "Crafting Interpreters - Robert Nystrom",
        "desc": "Tree-walk interpreters, bytecode virtual machines, Pratt parsing, garbage collection, and stack-based execution architectures.",
        "triggers": ["nystrom", "crafting-interpreters", "bytecode-vm", "pratt-parsing", "tree-walk-interpreter", "garbage-collection"],
        "foundations": [
            "Pratt Parsing (Top-Down Operator Precedence): Associating parse functions with token types and binding powers to parse expressions cleanly in O(N).",
            "Stack-Based VM Dispatch: Executing instructions via a central bytecode evaluation loop manipulating an explicit operand value stack.",
            "Mark-and-Sweep Garbage Collection: Tracing reachable objects from root references (stack, globals) and reclaiming unreachable memory.",
        ],
        "protocol": "Scaffold internal domain-specific scripting interpreters using Pratt parsing for ergonomic expressions and stack-based bytecode evaluation for execution speed.",
        "anti_patterns": [
            "Writing messy recursive-descent parsers for mathematical expressions when Pratt parsing handles precedence cleanly.",
            "Creating circular object references in custom interpreters without cycle-collection support.",
        ],
    },
    {
        "id": "agentic-fowler-domain-specific-languages",
        "book": "Domain-Specific Languages - Martin Fowler",
        "desc": "Internal vs external DSLs, semantic models, fluent interfaces, parser combinators, and language workbenches for business rule modeling.",
        "triggers": ["fowler", "domain-specific-languages", "dsl-design", "fluent-interface", "semantic-model", "internal-dsl", "external-dsl"],
        "foundations": [
            "Semantic Model Decoupling: The DSL syntax (internal builder or external script) populates a pure, syntax-agnostic semantic object graph.",
            "Fluent Interface Protocol: Method chaining designed so sentences read as natural human language while remaining syntactically valid in host language.",
            "Grammar-Driven External DSL: When business domain rules need to be edited by non-programmers without recompiling application binaries.",
        ],
        "protocol": "Create clean internal DSLs with fluent builders for complex configurations. Keep the underlying semantic model strictly decoupled from the syntax layer.",
        "anti_patterns": [
            "Coupling DSL parsing logic directly with execution side-effects instead of building a semantic model first.",
            "Creating clunky, unreadable method chaining that defeats the purpose of a fluent interface.",
        ],
    },
    {
        "id": "agentic-parr-antlr4-grammar-dsl",
        "book": "The Definitive ANTLR 4 Reference - Terence Parr",
        "desc": "ALL(*) adaptive LL grammar parsing, listener vs visitor AST traversal patterns, lexical modes, and grammar ambiguity resolution.",
        "triggers": ["parr", "antlr4", "adaptive-ll-star", "ast-visitor", "ast-listener", "lexical-modes", "grammar-engineering"],
        "foundations": [
            "ALL(*) Parsing Algorithm: Dynamically explores lookahead paths at runtime using deterministic finite automata (DFA), handling complex grammar recursion.",
            "Visitor vs Listener Pattern: Listeners walk ASTs passively via event callbacks (enterRule/exitRule); Visitors explicitly control traversal order and return values.",
            "Lexical Mode Switching: Switching token rules contextually (e.g. entering string interpolation or embedded SQL blocks).",
        ],
        "protocol": "Generate robust language parsers using ANTLR4 grammars. Implement the Visitor pattern when traversing code structures for type checking and transpilation.",
        "anti_patterns": [
            "Introducing left-recursive grammar rules that cause infinite loops in non-adaptive parsers.",
            "Embedding arbitrary target language code actions directly into grammar files, destroying portability.",
        ],
    },
    {
        "id": "agentic-pierce-type-systems-soundness",
        "book": "Types and Programming Languages (TAPL) - Benjamin C. Pierce",
        "desc": "Simply typed lambda calculus, type safety (progress and preservation theorems), subtyping, parametric polymorphism, and Curry-Howard isomorphism.",
        "triggers": ["pierce", "tapl", "type-systems", "type-soundness", "progress-preservation", "lambda-calculus", "curry-howard"],
        "foundations": [
            "Type Safety = Progress + Preservation: Progress: A well-typed term is either a value or can take an evaluation step. Preservation: If t : T and t -> t', then t' : T.",
            "Curry-Howard Isomorphism: Types correspond to logical propositions; programs correspond to proofs of those propositions.",
            "Subtyping Invariant (Liskov): S <: T means any term of type S can be safely used in a context expecting type T.",
        ],
        "protocol": "Leverage rich static type systems (Rust, TypeScript) to encode business invariants into types. Make illegal states unrepresentable at compile time.",
        "anti_patterns": [
            "Using stringly-typed or unstructured `any` types that bypass compiler safety verification.",
            "Violating the preservation theorem by writing unsafe casts that cause runtime type crashes.",
        ],
    },
    {
        "id": "agentic-harper-practical-foundations-pl",
        "book": "Practical Foundations for Programming Languages - Robert Harper",
        "desc": "Abstract binding trees, structural operational semantics, inductive definitions, dynamic dispatch vs static typing, and language modularity.",
        "triggers": ["harper", "pfpl", "operational-semantics", "abstract-binding-trees", "inductive-definitions", "type-theory"],
        "foundations": [
            "Structural Operational Semantics (SOS): Defining computation steps via inductive inference rules over abstract syntax terms.",
            "Abstract Binding Trees (ABTs): Enriching ASTs with formal variable binding, alpha-equivalence, and capture-avoiding substitution.",
            "Static/Dynamic Phase Distinction: Strict separation between compile-time static analysis and runtime dynamic evaluation.",
        ],
        "protocol": "Define language extensions and domain primitives using rigorous operational semantics. Enforce capture-avoiding substitution in code generation templates.",
        "anti_patterns": [
            "Naive macro expansions that cause variable name collisions (accidental variable capture).",
            "Blurring the phase distinction by executing dynamic runtime logic during static build steps.",
        ],
    },
    {
        "id": "agentic-sicp-evaluator-metacircular",
        "book": "Structure and Interpretation of Computer Programs - Harold Abelson & Gerald Jay Sussman",
        "desc": "Metacircular evaluators, homoiconicity, lexical closures, higher-order functional abstractions, stream processing, and lazy evaluation.",
        "triggers": ["sicp", "abelson-sussman", "metacircular-evaluator", "homoiconicity", "lexical-closures", "higher-order-functions", "lazy-evaluation"],
        "foundations": [
            "The Eval-Apply Cycle: Eval evaluates expressions relative to an environment; Apply applies procedures to arguments, closing the metacircular loop.",
            "Lexical Closures: Functions capture their enclosing environment bindings at definition time, maintaining state without global mutations.",
            "Streams as Infinite Data Structures: Decoupling the simulation of time from the order of events using delayed evaluation (lazy memoization).",
        ],
        "protocol": "Harness higher-order abstractions and closures to build modular agent middleware. Use lazy stream evaluation to process massive code bases incrementally.",
        "anti_patterns": [
            "Relying on mutable global variables rather than pure functional closures.",
            "Eagerly loading massive files into memory when streaming generators avoid out-of-memory errors.",
        ],
    },

    # =========================================================================
    # Cluster 7: Verification, Testing, Evals, Safety & Red Teaming (61-70)
    # =========================================================================
    {
        "id": "agentic-amodei-concrete-ai-safety",
        "book": "Concrete Problems in AI Safety - Dario Amodei et al.",
        "desc": "Avoiding negative side effects, reward hacking mitigation, scalable oversight, safe exploration, and robustness to distributional shift.",
        "triggers": ["amodei", "ai-safety", "reward-hacking", "negative-side-effects", "scalable-oversight", "safe-exploration", "distributional-shift"],
        "foundations": [
            "Side Effect Invariant: Actions must not cause unintended destructive perturbations to external systems outside the primary objective.",
            "Reward Hacking Defense: Ensure agent fitness cannot be maximized by trivial shortcuts (e.g. deleting failing tests).",
            "Safe Exploration: Constraining exploratory actions to verified sandboxes where catastrophic damage is mathematically impossible.",
        ],
        "protocol": "Sandbox all agent filesystem and shell operations. Protect test files from unauthorized tampering and enforce least-privilege security policies.",
        "anti_patterns": [
            "Permitting agents to edit the very test suites verifying their correctness.",
            "Running unverified agent shell commands directly on host production machines.",
        ],
    },
    {
        "id": "agentic-hendrycks-benchmarking-evals",
        "book": "Measuring Massive Multitask Language Understanding (MMLU) and Benchmarking - Dan Hendrycks et al.",
        "desc": "Benchmark design, multi-choice evaluation rubrics, calibration curves, normalized scoring, and contamination/leakage detection.",
        "triggers": ["hendrycks", "mmlu", "llm-benchmarking", "evaluation-rubrics", "calibration-curves", "contamination-detection"],
        "foundations": [
            "Normalized Scoring: Evaluating agents against standardized multidimensional benchmark suites across zero-shot and few-shot splits.",
            "Model Calibration: Confidence scores must reflect true empirical accuracy: E_{(X, Y)}[|P(Y=y | P_pred=p) - p|] -> 0.",
            "Data Leakage Auditing: Ensuring benchmark evaluation tasks are not present in the agent's pre-training or fine-tuning datasets.",
        ],
        "protocol": "Establish rigorous internal evals for code generation agents. Track pass@1 and pass@k across diverse coding tasks to detect regression before deploying.",
        "anti_patterns": [
            "Evaluating agents only on synthetic tasks identical to prompt examples.",
            "Relying on subjective human vibes without quantitative automated benchmark metrics.",
        ],
    },
    {
        "id": "agentic-perez-red-teaming-adversarial",
        "book": "Red Teaming Language Models with Language Models - Ethan Perez et al.",
        "desc": "Automated red-teaming, prompt injection vulnerability discovery, jailbreak fuzzing, adversarial perturbation testing, and safety alignment.",
        "triggers": ["perez", "red-teaming", "adversarial-testing", "prompt-injection", "jailbreak-fuzzing", "automated-redteaming"],
        "foundations": [
            "Adversarial Fuzzing Loop: Using an attacker LLM to generate perturbations designed to trigger safety violations or system prompt leakage.",
            "Zero-Tolerance Injection Guard: Verifying that user input payloads cannot override system-level safety instructions.",
            "Robustness Under Perturbation: Output behavior must remain sound despite whitespace noise, homoglyphs, or semantic trickery.",
        ],
        "protocol": "Subject all production agent prompts to automated red-teaming sweeps. Test injection vectors against tool call parameters and file editing commands.",
        "anti_patterns": [
            "Deploying agents without testing against prompt injection attacks.",
            "Assuming trust in external user inputs, comments in scraped code, or PR descriptions.",
        ],
    },
    {
        "id": "agentic-anthropic-constitutional-ai",
        "book": "Constitutional AI: Harmlessness from AI Feedback - Yuntao Bai et al.",
        "desc": "Principle-based self-critique, Reinforcement Learning from AI Feedback (RLAIF), constitutional rulesets, and automated chain-of-thought moderation.",
        "triggers": ["anthropic", "constitutional-ai", "rlaif", "self-critique", "constitutional-rules", "chain-of-thought-moderation"],
        "foundations": [
            "Critique and Revision Loop: Generate response -> Critique response against constitution principles -> Revise response to satisfy principles.",
            "Constitutional Principles: Unambiguous axioms governing safety, copyright, ethical behavior, and software correctness.",
            "RLAIF Alignment: Training preference models using automated AI critiques based on constitutional criteria rather than manual human labeling.",
        ],
        "protocol": "Equip coding agents with an explicit architectural constitution. Before committing code, have the agent execute a self-critique step against the constitution.",
        "anti_patterns": [
            "Committing raw first-draft code without a critique and revision pass.",
            "Vague constitutional rules that cannot be objectively verified by automated checks.",
        ],
    },
    {
        "id": "agentic-ozkaya-llm-software-evals",
        "book": "LLMs in Software Engineering: Evaluation & Verification - Ipek Ozkaya",
        "desc": "SWE-bench decomposition, pass@k metrics, patch verification, test suite execution in sandboxes, and regression prevention in agentic software engineering.",
        "triggers": ["ozkaya", "swe-bench", "software-evals", "pass-at-k", "patch-verification", "regression-prevention", "sandbox-execution"],
        "foundations": [
            "Pass@k Metric: Probability that at least one of k generated code samples passes all unit tests: pass@k = E[1 - comb(n - c, k) / comb(n, k)].",
            "Isolated Sandbox Verification: Compiling and executing generated patches inside ephemeral Docker or WebAssembly containers.",
            "Regression Invariant: A patch is valid if and only if it makes previously failing tests pass without breaking any existing passing tests.",
        ],
        "protocol": "Evaluate agent-generated git patches inside isolated worktrees. Run full regression test suites before presenting solutions to the developer.",
        "anti_patterns": [
            "Accepting patches that resolve a local bug but introduce silent regressions elsewhere.",
            "Executing generated code directly on the host development machine without sandboxing.",
        ],
    },
    {
        "id": "agentic-beck-tdd-verifiable-contracts",
        "book": "Test-Driven Development: By Example - Kent Beck",
        "desc": "Red-Green-Refactor cycle, test-first specifications, triangulation, isolation through test doubles, and regression test suites.",
        "triggers": ["beck", "tdd", "test-driven-development", "red-green-refactor", "triangulation", "test-doubles", "verifiable-contracts"],
        "foundations": [
            "Red-Green-Refactor Invariant: 1. Write a failing test (Red). 2. Write minimal code to pass the test (Green). 3. Clean up design without breaking tests (Refactor).",
            "Triangulation: Generalize code logic only when you have two or more distinct examples/tests requiring that generalization.",
            "Isolation: Tests must run independently in any order without shared mutable state or environmental dependencies.",
        ],
        "protocol": "Direct agents to always write the unit test FIRST. The agent must verify the test fails with the expected error before writing production code to pass it.",
        "anti_patterns": [
            "Writing production code before tests, leading to untestable designs or confirmation-biased tests.",
            "Writing tests that pass trivially without actually exercising the targeted failure mode.",
        ],
    },
    {
        "id": "agentic-claessen-property-based-testing",
        "book": "QuickCheck: A Lightweight Tool for Random Testing of Haskell Programs - Koen Claessen & John Hughes",
        "desc": "Property-based testing, universal property specifications, algebraic invariants (associativity, idempotence, round-trip), and automated generative testing.",
        "triggers": ["claessen", "hughes", "quickcheck", "property-based-testing", "algebraic-invariants", "generative-testing", "round-trip-testing"],
        "foundations": [
            "Universal Property Invariant: forall x in Domain: Property(x) == true across thousands of randomly generated inputs.",
            "Round-Trip Property: deserialize(serialize(x)) == x for all valid domain objects x.",
            "Idempotence Property: f(f(x)) == f(x) for operations like formatting, normalization, and reconciliation.",
        ],
        "protocol": "Implement property-based tests (using proptest, hypothesis, or QuickCheck) for all serialization, parsers, and mathematical state transitions.",
        "anti_patterns": [
            "Relying solely on 2-3 hardcoded example test cases for complex parsers or data serializers.",
            "Writing property tests with weak assertions that never challenge edge cases.",
        ],
    },
    {
        "id": "agentic-maciver-invariant-shrinking",
        "book": "Hypothesis: Modern Property-Based Testing and Invariant Shrinking - David R. MacIver",
        "desc": "Automated minimal test case reduction (shrinking), stateful model-based testing, falsification search, and integration fuzzing.",
        "triggers": ["maciver", "hypothesis", "test-shrinking", "minimal-reproduction", "stateful-testing", "falsification-search"],
        "foundations": [
            "Minimal Counterexample Shrinking: When a test fails on complex input X, automatically shrink X down to the smallest minimal failing reproduction.",
            "Stateful Model-Based Testing: Execute randomized sequences of state-machine actions comparing system state against an abstract reference model.",
            "Deterministic Replay: Any shrunk counterexample must reproduce identically given its random seed.",
        ],
        "protocol": "When debugging, have the agent automatically shrink failing test payloads to minimal 1-line reproductions before attempting code fixes.",
        "anti_patterns": [
            "Dumping massive 1000-line failure logs on the developer without isolating the minimal failing input.",
            "Non-deterministic tests that cannot be reliably reproduced from a fixed seed.",
        ],
    },
    {
        "id": "agentic-clarke-model-checking-invariants",
        "book": "Model Checking - Edmund M. Clarke, Orna Grumberg & Doron A. Peled",
        "desc": "State transition systems, temporal logic formulas, safety and liveness properties, state-space explosion mitigation, Binary Decision Diagrams (BDD).",
        "triggers": ["clarke", "grumberg", "peled", "model-checking", "temporal-logic", "safety-liveness", "state-space-verification"],
        "foundations": [
            "Safety vs Liveness: Safety: 'Bad things never happen' (G ~bad). Liveness: 'Good things eventually happen' (F good).",
            "Kripke Structure Formalization: M = (S, S_0, R, L), verifying whether M satisfies temporal formula phi (M |= phi).",
            "Counterexample Generation: Model checkers provide exact execution traces demonstrating how an invariant is violated.",
        ],
        "protocol": "Verify critical concurrency locks, consensus protocols, and state machines against formal safety and liveness invariants.",
        "anti_patterns": [
            "Confusing safety with liveness, ignoring deadlock states where no bad state occurs but progress ceases.",
            "State explosions caused by modeling unconstrained integers instead of bounded abstractions.",
        ],
    },
    {
        "id": "agentic-baier-temporal-logic-ltl-ctl",
        "book": "Principles of Model Checking - Christel Baier & Joost-Pieter Katoen",
        "desc": "Linear Temporal Logic (LTL), Computation Tree Logic (CTL), Büchi automata, bisimulation equivalence, and probabilistic model checking.",
        "triggers": ["baier", "katoen", "principles-model-checking", "ltl", "ctl", "buchi-automata", "bisimulation-equivalence"],
        "foundations": [
            "LTL Operators: Next (X), Globally/Always (G), Finally/Eventually (F), Until (U) over infinite execution paths.",
            "Büchi Automata Translation: An LTL formula is converted into a non-deterministic Büchi automaton accepting infinite words that violate the property.",
            "Bisimulation Equivalence: Systems S_1 and S_2 are bisimilar (S_1 ~ S_2) if they can simulate each other's transitions step-by-step.",
        ],
        "protocol": "Formulate critical async system invariants in LTL. Verify that every requested background task eventually terminates or reports an error.",
        "anti_patterns": [
            "Creating asynchronous event loops with no eventual termination or cancellation guarantee.",
            "Assuming path-based linear properties hold across branching computation trees without CTL checks.",
        ],
    },

    # =========================================================================
    # Cluster 8: Distributed Systems, Protocols, Event-Driven & Tool Interop (71-80)
    # =========================================================================
    {
        "id": "agentic-kleppmann-distributed-consistency",
        "book": "Designing Data-Intensive Applications - Martin Kleppmann",
        "desc": "ACID vs BASE, linearizability, eventual consistency, leader-follower replication, partitioning, distributed transactions, and event sourcing.",
        "triggers": ["kleppmann", "ddia", "data-intensive-applications", "linearizability", "eventual-consistency", "event-sourcing", "distributed-consensus"],
        "foundations": [
            "CAP Theorem & Trade-offs: In the presence of a network partition (P), a distributed system must choose between Consistency (C) or Availability (A).",
            "Linearizability: All operations appear to execute atomically at a single instant in time between their invocation and response.",
            "Two-Phase Commit (2PC) Vulnerability: Coordinator failure during the prepare/commit window blocks participants indefinitely.",
        ],
        "protocol": "Design agent persistent stores with clear consistency guarantees. Use event-sourced logs for auditability and idempotent operations for safe retries.",
        "anti_patterns": [
            "Assuming network calls never fail or time out, omitting retry backoff and circuit breakers.",
            "Relying on distributed locks without fencing tokens, causing split-brain storage writes.",
        ],
    },
    {
        "id": "agentic-tanenbaum-distributed-systems",
        "book": "Distributed Systems: Principles and Paradigms - Andrew S. Tanenbaum & Maarten van Steen",
        "desc": "RPC protocols, message-oriented middleware, distributed naming, synchronization, fault tolerance, and process migration.",
        "triggers": ["tanenbaum", "van-steen", "distributed-systems", "remote-procedure-call", "rpc-protocols", "fault-tolerance", "distributed-naming"],
        "foundations": [
            "Fallacies of Distributed Computing: The network is reliable; latency is zero; bandwidth is infinite; the network is secure; topology doesn't change.",
            "Idempotent Remote Procedure Calls: Remote operations must be idempotent so retransmissions do not cause duplicate side effects: f(f(x)) == f(x).",
            "Heartbeat & Lease Heartbeats: Detecting node failure using periodic heartbeats with bounded timeout thresholds.",
        ],
        "protocol": "Wrap all inter-agent RPCs in idempotent envelopes with unique idempotency keys. Implement exponential backoff and jitter on network retries.",
        "anti_patterns": [
            "Treating remote API calls as synchronous local method calls, ignoring latency and network partitions.",
            "Non-idempotent endpoints that double-charge or create duplicate records on network retry.",
        ],
    },
    {
        "id": "agentic-lamport-logical-clocks",
        "book": "Time, Clocks, and the Ordering of Events in a Distributed System - Leslie Lamport",
        "desc": "Partial orderings, happens-before relation (->), logical timestamps, vector clocks, total ordering consistency, and distributed state machines.",
        "triggers": ["lamport", "logical-clocks", "happens-before", "vector-clocks", "lamport-timestamps", "distributed-ordering"],
        "foundations": [
            "Happens-Before Relation (->): If a and b are in the same process and a occurs before b, then a -> b. If a is send and b is receive, a -> b.",
            "Lamport Timestamp Update: C(e) = max(C_local, C_msg) + 1, establishing a strict partial order across distributed events.",
            "Vector Clocks: V_i[j] tracks agent i's knowledge of agent j's logical time, enabling detection of causal vs concurrent events.",
        ],
        "protocol": "Order agent swarm actions and messages using Lamport timestamps or vector clocks to guarantee causal consistency without relying on unsynchronized wall clocks.",
        "anti_patterns": [
            "Using system physical wall clocks (SystemTime) to order distributed events, causing clock drift corruption.",
            "Assuming concurrent events have a natural causal order without vector clock verification.",
        ],
    },
    {
        "id": "agentic-ongaro-raft-distributed-consensus",
        "book": "In Search of an Understandable Consensus Algorithm (Raft) - Diego Ongaro & John Ousterhout",
        "desc": "Leader election, log replication, safety invariants, randomized election timeouts, joint consensus reconfiguration, and state machine replication.",
        "triggers": ["ongaro", "ousterhout", "raft-consensus", "leader-election", "log-replication", "state-machine-replication", "randomized-timeouts"],
        "foundations": [
            "Raft State Invariants: Election Safety (at most one leader per term); Leader Append-Only; Log Matching; Leader Completeness; State Machine Safety.",
            "Quorum Majority Rule: A leader can commit a log entry only after it is replicated on a strict majority of nodes: floor(N/2) + 1.",
            "Randomized Election Timeouts: Split-vote prevention by randomizing election timeouts (e.g. 150ms-300ms).",
        ],
        "protocol": "Implement leader election and state machine replication for multi-agent clusters using Raft consensus. Guarantee quorum agreement before committing configuration changes.",
        "anti_patterns": [
            "Split-brain scenarios caused by committing log entries without majority quorum confirmation.",
            "Fixed election timeouts causing perpetual split-vote election ties.",
        ],
    },
    {
        "id": "agentic-hohpe-enterprise-integration-patterns",
        "book": "Enterprise Integration Patterns: Designing, Building, and Deploying Messaging Solutions - Gregor Hohpe & Bobby Woolf",
        "desc": "Message channels, pipes and filters, content-based router, scatter-gather, message translator, idempotent receiver, and pub/sub architectures.",
        "triggers": ["hohpe", "woolf", "enterprise-integration", "pipes-and-filters", "content-based-router", "scatter-gather", "message-translator", "idempotent-receiver"],
        "foundations": [
            "Pipes and Filters Architecture: Decomposing complex data processing into independent, reusable processing stages connected by message pipes.",
            "Content-Based Router: Inspecting message payload attributes to dynamically direct messages to the appropriate downstream agent specialist.",
            "Scatter-Gather Pattern: Broadcasting a query to multiple agent workers and aggregating/ranking their responses into a single composite output.",
        ],
        "protocol": "Design multi-agent processing pipelines using Enterprise Integration Patterns: Scatter-Gather for parallel research, Content-Based Routers for language dispatch.",
        "anti_patterns": [
            "Direct point-to-point spaghetti coupling between agents without message channels.",
            "Non-idempotent message consumers that corrupt state upon receiving duplicate delivery.",
        ],
    },
    {
        "id": "agentic-newman-microservices-tool-isolation",
        "book": "Building Microservices: Designing Fine-Grained Systems - Sam Newman",
        "desc": "Loose coupling, high cohesion, bounded contexts, API versioning, canary deployments, circuit breakers, and sandboxed tool isolation.",
        "triggers": ["newman", "building-microservices", "service-isolation", "bounded-contexts", "circuit-breakers", "canary-deployment"],
        "foundations": [
            "High Cohesion & Loose Coupling: Code that changes together stays together; services know as little as possible about each other's internals.",
            "Circuit Breaker Pattern: Automatically tripping open to stop calling a failing dependency, returning fast fallbacks rather than cascading failures.",
            "Backwards-Compatible API Versioning: Tolerant reader pattern ensuring schema additions do not break existing downstream service clients.",
        ],
        "protocol": "Isolate agent tool execution within microservice boundaries with strict circuit breakers and timeouts. Protect production backends from cascading agent retries.",
        "anti_patterns": [
            "Cascading failures where a single failing agent tool crashes the entire orchestrator.",
            "Breaking API contract changes that break downstream client agents.",
        ],
    },
    {
        "id": "agentic-richards-software-architecture-tradeoffs",
        "book": "Fundamentals of Software Architecture - Mark Richards & Neal Ford",
        "desc": "Architectural characteristics (-ilities), modularity, component coupling, trade-off analysis, fitness functions, and architecture governance.",
        "triggers": ["richards", "ford", "software-architecture", "architectural-tradeoffs", "fitness-functions", "component-coupling", "architecture-governance"],
        "foundations": [
            "First Law of Software Architecture: Everything in software architecture is a trade-off (performance vs simplicity, flexibility vs maintainability).",
            "Automated Architectural Fitness Functions: Automated tests that execute in CI/CD to verify architecture characteristics (e.g. cycle detection, module coupling).",
            "Connascence Metrics: Measuring the strength of coupling between components to minimize ripple effects of changes.",
        ],
        "protocol": "Formulate architectural decisions as explicit trade-offs. Protect system structure by writing automated fitness functions that prevent circular dependencies.",
        "anti_patterns": [
            "Claiming an architectural choice has no downsides, ignoring hidden operational or latency costs.",
            "Allowing architectural degradation over time due to lack of automated fitness functions.",
        ],
    },
    {
        "id": "agentic-fielding-rest-agent-apis",
        "book": "Architectural Styles and the Design of Network-based Software Architectures (REST) - Roy Thomas Fielding",
        "desc": "Statelessness, uniform interface, cacheability, layered systems, HATEOAS, and resource-oriented modeling for autonomous agent APIs.",
        "triggers": ["fielding", "rest-architecture", "hateoas", "uniform-interface", "statelessness", "resource-oriented", "web-architecture"],
        "foundations": [
            "Statelessness Invariant: Each request from client to server must contain all the information necessary to understand and process the request.",
            "Uniform Interface (HATEOAS): Hypermedia as the Engine of Application State; clients transition through states via hypermedia links in responses.",
            "Cacheability Constraint: Responses must explicitly define themselves as cacheable or non-cacheable to optimize network efficiency.",
        ],
        "protocol": "Design agent-accessible web APIs using strict REST principles. Provide self-descriptive hypermedia links in API responses so agents can discover available actions.",
        "anti_patterns": [
            "Maintaining hidden session state on servers that breaks client agent failover and scalability.",
            "Tunneling arbitrary non-idempotent operations through HTTP GET requests.",
        ],
    },
    {
        "id": "agentic-henning-rpc-schema-contracts",
        "book": "Advanced CORBA / Modern RPC & Protocol Buffers - Michi Henning & Steve Vinoski",
        "desc": "Interface Definition Languages (IDL), serialization efficiency, binary schemas, backwards compatibility, and strongly typed RPC contracts.",
        "triggers": ["henning", "vinoski", "rpc-contracts", "protocol-buffers", "interface-definition-language", "binary-serialization", "schema-evolution"],
        "foundations": [
            "IDL Contract Primacy: The schema is the single source of truth; client and server stubs are mechanically generated from the IDL.",
            "Binary Wire Efficiency: Protocol Buffers / Cap'n Proto binary serialization delivers order-of-magnitude faster throughput and smaller footprints than JSON.",
            "Tag-Based Backwards Compatibility: Fields are identified by field numbers/tags; unknown fields are preserved, enabling zero-downtime schema evolution.",
        ],
        "protocol": "Use strongly typed schemas (Protocol Buffers, Cap'n Proto, or JSON Schema) for all high-throughput agent-to-agent and tool communications.",
        "anti_patterns": [
            "Passing loosely typed, undocumented JSON dictionaries between distributed agent services.",
            "Changing field IDs or deleting fields in active schemas without migration paths.",
        ],
    },
    {
        "id": "agentic-mcp-protocol-specification",
        "book": "Anthropic Model Context Protocol Specification - Anthropic MCP Architecture",
        "desc": "Client-Host-Server topology, JSON-RPC 2.0 framing, resource subscriptions, tool invocation contracts, prompt templates, and security sandboxing.",
        "triggers": ["mcp", "model-context-protocol", "json-rpc", "mcp-server", "mcp-client", "tool-invocation", "resource-subscriptions"],
        "foundations": [
            "Client-Host-Server Architecture: Host application (e.g. Tagisan) coordinates MCP Clients that connect to isolated MCP Servers providing tools and resources.",
            "JSON-RPC 2.0 Framing: Strict request, response, notification, and error objects with deterministic error codes (-32600 to -32603).",
            "Resource URI Schemes: Resources identified by standardized URIs (e.g. `file:///`, `postgres://`) with subscription notifications on content changes.",
        ],
        "protocol": "Expose all agent capabilities, tools, and project contexts through the Model Context Protocol (MCP). Enforce strict parameter validation on all incoming tool calls.",
        "anti_patterns": [
            "Writing proprietary ad-hoc tool execution protocols when standard MCP provides universal interop.",
            "Failing to validate tool input arguments against the declared JSON schema before execution.",
        ],
    },

    # =========================================================================
    # Cluster 9: Cognitive Architectures, Metacognition & Self-Improving Systems (81-90)
    # =========================================================================
    {
        "id": "agentic-laird-soar-cognitive-architecture",
        "book": "The Soar Cognitive Architecture - John E. Laird",
        "desc": "Production system rules, working memory elements (WMEs), subgoaling on impasses, chunking (rule learning), and unified cognitive architectures.",
        "triggers": ["laird", "soar-architecture", "cognitive-architecture", "subgoaling-impasses", "chunking-learning", "working-memory-elements"],
        "foundations": [
            "Decision Cycle: Input -> Elaboration (parallel production firing) -> Operator Proposal -> Operator Selection -> Operator Application -> Output.",
            "Subgoaling on Impasses: When the agent cannot select an operator (tie, conflict, or no-change impasse), Soar creates a substate to resolve it.",
            "Chunking Invariant: The system compiles the results of successful substate problem solving into new permanent production rules.",
        ],
        "protocol": "Detect reasoning impasses (e.g. uncertainty between two libraries). Spawn an isolated subagent to resolve the impasse, then cache the resolution rule.",
        "anti_patterns": [
            "Failing to recognize when an agent is in an impasse, causing endless circular retries.",
            "Discarding the lessons learned from resolved impasses instead of chunking them into memory.",
        ],
    },
    {
        "id": "agentic-minsky-society-of-mind",
        "book": "The Society of Mind - Marvin Minsky",
        "desc": "Mind as a society of mindless agents, agency hierarchies, censors and suppressors, cross-exclusion, and emergence of intelligence from simple modules.",
        "triggers": ["minsky", "society-of-mind", "censors-suppressors", "agency-hierarchies", "cross-exclusion", "emergent-intelligence"],
        "foundations": [
            "Agency Composition: Intelligence emerges from the interactions of many simple agents, none of which are intelligent on their own.",
            "Censors & Suppressors: Specialized agents whose sole function is inhibiting destructive thoughts or forbidden actions before they execute.",
            "Cross-Exclusion: Competing agencies mutually inhibit one another so only one dominant action plan gains motor control at a time.",
        ],
        "protocol": "Implement specialized censor agents (e.g. security validator, style linter) that inspect and veto actions proposed by generative coding agents.",
        "anti_patterns": [
            "Attempting to build an all-knowing monolithic agent instead of a society of focused specialists.",
            "Allowing competing agent plans to execute simultaneously without cross-exclusion.",
        ],
    },
    {
        "id": "agentic-kahneman-dual-process-thinking",
        "book": "Thinking, Fast and Slow - Daniel Kahneman",
        "desc": "System 1 (heuristic, rapid, intuitive) vs System 2 (deliberative, analytical, slow), cognitive biases, anchoring, loss aversion, and metacognition.",
        "triggers": ["kahneman", "thinking-fast-and-slow", "system-1-system-2", "dual-process-theory", "cognitive-biases", "deliberative-reasoning"],
        "foundations": [
            "Dual-Process Architecture: System 1 operates automatically and quickly with little or no effort; System 2 allocates attention to effortful mental operations.",
            "Substitution Heuristic: When faced with a difficult question, System 1 substitutes an easier question without the system noticing.",
            "Anchoring & Confirmation Bias: The tendency to over-rely on the first piece of information encountered and seek confirming evidence.",
        ],
        "protocol": "Use fast System 1 generation for routine boilerplate syntax, but invoke explicit deliberative System 2 verification before committing architecture or security changes.",
        "anti_patterns": [
            "Letting System 1 handle safety-critical concurrency logic, yielding subtle race conditions.",
            "Over-thinking simple formatting tasks with heavyweight System 2 reasoning chains.",
        ],
    },
    {
        "id": "agentic-hofstadter-strange-loops-recursion",
        "book": "Gödel, Escher, Bach: An Eternal Golden Braid - Douglas R. Hofstadter",
        "desc": "Strange loops, self-referential systems, meta-reasoning, recursive isomorphism, Gödelian limits of formal systems, and consciousness emergence.",
        "triggers": ["hofstadter", "geb", "strange-loops", "self-reference", "meta-reasoning", "recursive-isomorphism", "godel-incompleteness"],
        "foundations": [
            "Strange Loop Phenomenon: A paradoxical hierarchy where moving up through levels unexpectedly brings one back to the starting point.",
            "Gödelian Incompleteness: Any consistent formal system capable of expressing arithmetic contains truths that cannot be proven within the system.",
            "Level Jumping (Jumping out of the system): The metacognitive ability of an agent to reflect on its own rules and modify its operational frame.",
        ],
        "protocol": "Enable agents to 'jump out of the system' when caught in infinite debugging loops: stop code editing and evaluate whether the underlying problem premise is flawed.",
        "anti_patterns": [
            "Getting trapped in recursive self-referential loops without termination bounds.",
            "Assuming a local formal system or test framework is complete and incapable of subtle bugs.",
        ],
    },
    {
        "id": "agentic-simon-bounded-rationality-heuristics",
        "book": "The Sciences of the Artificial (3rd ed) - Herbert A. Simon",
        "desc": "Bounded rationality, satisficing vs optimizing, near-decomposability of complex systems, and cognitive architecture heuristics.",
        "triggers": ["simon", "herbert-simon", "bounded-rationality", "satisficing", "near-decomposability", "sciences-of-the-artificial"],
        "foundations": [
            "Bounded Rationality: Decision-makers lack the cognitive resources and complete information required to find mathematically optimal solutions.",
            "Satisficing Principle: Select the first candidate solution that meets or exceeds predefined aspiration thresholds rather than searching for the global optimum.",
            "Near-Decomposability: Complex systems can be decomposed into subsystems whose internal interactions are much stronger than interactions between subsystems.",
        ],
        "protocol": "Apply satisficing criteria to code synthesis: accept solutions that pass all tests and meet performance budgets without endlessly striving for theoretical perfection.",
        "anti_patterns": [
            "Paralysis by analysis: searching indefinitely for the 'perfect' algorithm when a standard solution is 100% adequate.",
            "Monolithic coupling that destroys the near-decomposability of software modules.",
        ],
    },
    {
        "id": "agentic-newell-unified-cognition",
        "book": "Unified Theories of Cognition - Allen Newell",
        "desc": "Time scales of human action (biological, cognitive, rational, social bands), problem space hypothesis, and cognitive architecture benchmarks.",
        "triggers": ["newell", "unified-theories-cognition", "problem-space-hypothesis", "cognitive-bands", "cognitive-benchmarks", "action-timescales"],
        "foundations": [
            "Bands of Human Action: Biological (~1-10ms), Cognitive (~100ms-10s), Rational (~minutes-hours), and Social (~days-months) time scales.",
            "Problem Space Hypothesis: All goal-oriented cognitive behavior occurs through search within formulated problem spaces.",
            "Knowledge Level Principle: An agent's behavior can be predicted solely by knowing its goals and the knowledge it possesses.",
        ],
        "protocol": "Structure agent tasks across appropriate time scales: sub-second linter fixes at the cognitive band; multi-hour architectural refactors at the rational band.",
        "anti_patterns": [
            "Applying multi-hour deliberative planning to millisecond-level syntax completions.",
            "Failing to track overarching rational-band goals during low-level cognitive debugging.",
        ],
    },
    {
        "id": "agentic-sun-clarion-implicit-explicit",
        "book": "An Introduction to Dual-Process Cognitive Systems with CLARION - Ron Sun",
        "desc": "Implicit vs explicit cognitive processes, bottom-up learning, motivational subsystems, and metacognitive control loops in hybrid agents.",
        "triggers": ["sun", "clarion", "implicit-explicit-learning", "dual-process-cognition", "motivational-subsystem", "metacognitive-control"],
        "foundations": [
            "Two-Level Cognitive Architecture: Bottom level encodes implicit procedural skills (neural weights); top level encodes explicit declarative rules (symbolic logic).",
            "Bottom-Up Learning: Explicit symbolic rules are extracted from successful implicit neural trials through rule extraction algorithms.",
            "Metacognitive Regulation: Monitoring cognitive progress, adjusting reinforcement learning rates, and balancing exploration vs exploitation.",
        ],
        "protocol": "Combine implicit neural pattern matching (LLM generation) with explicit symbolic verification (linters, compilers, formal provers) in every agent turn.",
        "anti_patterns": [
            "Relying solely on implicit neural intuition without explicit symbolic verification.",
            "Rigid symbolic systems that cannot adapt to fuzzy or ambiguous natural language requirements.",
        ],
    },
    {
        "id": "agentic-lake-cognitive-concept-learning",
        "book": "Building Machines That Learn and Think Like People - Brenden M. Lake et al.",
        "desc": "Compositionality, causality, learning to learn (meta-learning), intuitive physics, intuitive psychology, and sample-efficient concept learning.",
        "triggers": ["lake", "concept-learning", "intuitive-psychology", "meta-learning", "compositional-concepts", "sample-efficiency"],
        "foundations": [
            "Compositionality Principle: Complex concepts are constructed as structured compositional programs built from primitive atomic elements.",
            "Causal Model Induction: Representing knowledge as causal generative models rather than surface statistical correlations.",
            "Learning-to-Learn (Meta-Learning): Accelerating new concept acquisition by transferring abstract structural schemas learned from previous tasks.",
        ],
        "protocol": "Structure code generation around composable building blocks. Induce causal models of the developer's intent rather than matching surface keywords.",
        "anti_patterns": [
            "Treating new software frameworks as completely alien instead of mapping them to known architectural abstractions.",
            "Superficial copy-pasting without understanding the underlying causal component relationships.",
        ],
    },
    {
        "id": "agentic-schmidhuber-intrinsic-curiosity",
        "book": "Formal Theory of Fun & Intrinsic Motivation in Self-Improving Agents - Jürgen Schmidhuber",
        "desc": "Compression progress as intrinsic reward, artificial curiosity, Gödel machines, self-invented problems, and mathematically optimal self-improvement.",
        "triggers": ["schmidhuber", "intrinsic-curiosity", "compression-progress", "godel-machine", "self-improving-agents", "formal-theory-fun"],
        "foundations": [
            "Compression Progress Reward: Intrinsic reward R_intrinsic(t) = C(state | model_{t-1}) - C(state | model_t), rewarding the discovery of compressible regularities.",
            "Gödel Machine Invariant: A self-referential system that rewires its own code if and only if it can formally prove the modification yields superior expected utility.",
            "Artificial Curiosity: Actively seeking out environments where the agent's current predictive model makes errors, accelerating learning.",
        ],
        "protocol": "Incentivize agents to proactively explore and refactor confusing, high-entropy legacy code modules. Reward the discovery of simplifying abstractions.",
        "anti_patterns": [
            "Allowing self-modifying agents to alter core safety invariants without mathematical proof of safety.",
            "Focusing exclusively on easy, familiar code while avoiding poorly understood mission-critical modules.",
        ],
    },
    {
        "id": "agentic-wang-nars-non-axiomatic-reasoning",
        "book": "Non-Axiomatic Logic: A Model of Intelligent Reasoning (NARS) - Pei Wang",
        "desc": "Assumption of Insufficient Knowledge and Resources (AIKR), truth values as frequency and confidence, syllogistic inference, and open-world reasoning.",
        "triggers": ["wang", "nars", "non-axiomatic-logic", "aikr", "truth-value-confidence", "syllogistic-reasoning", "open-world-reasoning"],
        "foundations": [
            "AIKR Axiom: An intelligent agent must adapt under the constraint of Insufficient Knowledge and Resources (finite memory, finite compute, real-time deadlines).",
            "NARS Truth Value Pair: <f, c>, where f in [0, 1] is the frequency of positive evidence, and c in [0, 1) is confidence based on total amount of evidence.",
            "Non-Axiomatic Syllogism: Deducing, inducting, and abducing relationships between concepts while tracking evidence confidence intervals.",
        ],
        "protocol": "Operate explicitly under AIKR constraints: prioritize high-confidence code suggestions when time is scarce; fall back to conservative approximations.",
        "anti_patterns": [
            "Assuming infinite time and compute to solve coding tasks under strict production deadlines.",
            "Treating uncertain empirical observations as binary (1 or 0) mathematical truths.",
        ],
    },

    # =========================================================================
    # Cluster 10: Human-AI Collaboration, Steering, Architecture & Evolution (91-100)
    # =========================================================================
    {
        "id": "agentic-shneiderman-human-centered-ai",
        "book": "Human-Centered AI - Ben Shneiderman",
        "desc": "High automation and high human control (HCAI matrix), explainable AI, reliable/safe/trustworthy design, and human oversight in agent systems.",
        "triggers": ["shneiderman", "human-centered-ai", "hcai-matrix", "human-in-the-loop", "high-automation-high-control", "explainable-ai"],
        "foundations": [
            "HCAI Two-Dimensional Matrix: Simultaneously maximize Human Control and Computer Automation (avoiding the false choice between total autonomy or manual toil).",
            "Continuous Oversight Interfaces: Dashboards providing real-time visibility into agent telemetry, pending actions, and override controls.",
            "Audit Trails & Explainability: Every autonomous action must produce an intelligible audit log explaining why the action was chosen.",
        ],
        "protocol": "Provide transparent audit previews before applying multi-file refactorings. Give the developer instant 1-click override and rollback capabilities.",
        "anti_patterns": [
            "Black-box autonomous modifications with zero explanation or user visibility.",
            "Degrading developer control in the name of full automation, breeding mistrust and rejection.",
        ],
    },
    {
        "id": "agentic-brooks-mythical-man-month",
        "book": "The Mythical Man-Month: Essays on Software Engineering - Frederick P. Brooks Jr.",
        "desc": "Brooks's Law, conceptual integrity, the surgical team, second-system effect, and essential vs accidental complexity in engineering projects.",
        "triggers": ["brooks", "mythical-man-month", "brooks-law", "conceptual-integrity", "surgical-team", "second-system-effect", "essential-complexity"],
        "foundations": [
            "Brooks's Law: Adding manpower to a late software project makes it later (due to combinatorial communication overhead: n*(n-1)/2).",
            "Conceptual Integrity: The most important attribute of software design; best achieved when a system reflects a single unified architectural vision.",
            "The Surgical Team: Structuring development teams around a chief architect/developer supported by specialized assistants (toolsmith, tester, editor).",
        ],
        "protocol": "Act as the chief architect's surgical team assistant. Preserve conceptual integrity across all modules and prevent communication bloat.",
        "anti_patterns": [
            "Spawning dozens of uncoordinated agent workers that create combinatorial git conflicts (Brooks's Law in multi-agent systems).",
            "Falling victim to the second-system effect by packing excessive bells and whistles into a redesign.",
        ],
    },
    {
        "id": "agentic-ousterhout-philosophy-software-design",
        "book": "A Philosophy of Software Design - John Ousterhout",
        "desc": "Deep modules vs shallow modules, information hiding, complexity as a symptom of dependency and obscurity, and strategic vs tactical programming.",
        "triggers": ["ousterhout", "philosophy-software-design", "deep-modules", "information-hiding", "tactical-tornado", "strategic-programming"],
        "foundations": [
            "Deep Module Invariant: The best modules provide powerful functionality through simple, compact interfaces (deep); avoid shallow modules.",
            "Strategic vs Tactical Programming: Tactical: quick patches that add technical debt; Strategic: investing 10-20% extra effort in clean design.",
            "Complexity Definition: Complexity is anything related to the structure of a software system that makes it hard to understand and modify.",
        ],
        "protocol": "Synthesize deep modules with simple public APIs that hide substantial internal complexity. Avoid shallow wrapper classes that increase obscurity.",
        "anti_patterns": [
            "Becoming a 'tactical tornado' that hacks in quick fixes while degrading overall codebase structure.",
            "Creating shallow interfaces that expose internal implementation details to callers.",
        ],
    },
    {
        "id": "agentic-martin-clean-architecture-boundaries",
        "book": "Clean Architecture: A Craftsman's Guide to Software Structure and Design - Robert C. Martin",
        "desc": "Dependency Inversion Principle, concentric architectural boundaries, entities, use cases, interface adapters, and framework independence.",
        "triggers": ["martin", "uncle-bob", "clean-architecture", "dependency-inversion", "concentric-boundaries", "use-cases", "framework-independence"],
        "foundations": [
            "The Dependency Rule: Source code dependencies must point only inward, toward higher-level policies: Entities -> Use Cases -> Adapters -> Frameworks.",
            "Entities & Business Logic Purity: Enterprise business rules must have zero dependencies on databases, UI frameworks, or external third-party libraries.",
            "Boundaries as Plugins: Databases and web delivery mechanisms are details that plug into the core application using interface ports.",
        ],
        "protocol": "Enforce concentric boundaries in generated code: keep business entities strictly decoupled from database engines and web frameworks via ports and adapters.",
        "anti_patterns": [
            "Importing database ORM entities directly into domain logic or UI views.",
            "Letting external framework conventions dictate core business domain models.",
        ],
    },
    {
        "id": "agentic-feathers-legacy-code-refactoring",
        "book": "Working Effectively with Legacy Code - Michael C. Feathers",
        "desc": "Legacy code definition (code without tests), sensing and separation pins, sprout/wrap method, characterization tests, and breaking dependencies.",
        "triggers": ["feathers", "legacy-code", "characterization-tests", "sprout-method", "wrap-method", "breaking-dependencies", "seams"],
        "foundations": [
            "Definition of Legacy Code: Code without unit tests. Tests are a safety harness allowing rapid, fearless modification without regressions.",
            "Characterization Tests: Tests that document and preserve the existing actual behavior of a legacy system before attempting refactoring.",
            "Seams: A place where you can alter behavior in a program without editing in that place (e.g. object seams, link seams).",
        ],
        "protocol": "Before modifying legacy code, write characterization tests to lock down current behavior. Use Sprout/Wrap methods to add new features safely.",
        "anti_patterns": [
            "Refactoring complex legacy code without first establishing automated regression tests.",
            "Assuming undocumented legacy behavior is a bug and removing it, breaking downstream clients.",
        ],
    },
    {
        "id": "agentic-forsgren-accelerate-dora-metrics",
        "book": "Accelerate: Building and Scaling High Performing Technology Organizations - Nicole Forsgren, Jez Humble & Gene Kim",
        "desc": "Four DORA metrics (Deployment Frequency, Lead Time for Changes, Change Failure Rate, Time to Restore Service), and continuous delivery practices.",
        "triggers": ["forsgren", "humble", "gene-kim", "accelerate", "dora-metrics", "continuous-delivery", "lead-time-for-changes"],
        "foundations": [
            "Four DORA Metrics: Deployment Frequency, Lead Time for Changes, Change Failure Rate (< 15%), Mean Time to Recovery (< 1 hour).",
            "Continuous Delivery Capabilities: Version control for all artifacts, trunk-based development, automated testing, loosely coupled architecture.",
            "Transformational Leadership: Empowering engineering teams with autonomous decision-making and psychological safety.",
        ],
        "protocol": "Optimize agent-assisted development to drive elite DORA metrics: sub-hour lead time from prompt to production, zero-regression trunk commits.",
        "anti_patterns": [
            "Long-lived feature branches that delay feedback and trigger painful merge conflicts.",
            "Deploying unverified changes that spike Change Failure Rates above healthy thresholds.",
        ],
    },
    {
        "id": "agentic-kim-phoenix-project-flow-theory",
        "book": "The Phoenix Project - Gene Kim, Kevin Behr & George Spafford",
        "desc": "The Three Ways (Flow, Feedback, Continual Learning), Theory of Constraints (Goldratt), work-in-progress (WIP) limits, and bottleneck management.",
        "triggers": ["kim", "phoenix-project", "three-ways", "theory-of-constraints", "wip-limits", "bottleneck-management", "devops-flow"],
        "foundations": [
            "The First Way (Principles of Flow): Accelerate the flow of work from Development to Operations; reduce batch sizes and WIP.",
            "The Second Way (Principles of Feedback): Create fast, reciprocal feedback loops from right to left; amplify feedback to prevent recurrence of errors.",
            "The Third Way (Continual Learning): Foster a culture of experimentation, calculated risk-taking, and learning from failure.",
        ],
        "protocol": "Identify the primary bottleneck in the software pipeline (e.g. slow tests, manual deploys) and subordinate all agent activities to resolving it.",
        "anti_patterns": [
            "Optimizing non-bottleneck stages, creating inventory piles without increasing throughput.",
            "Ignoring operational feedback and continuing to push code into broken environments.",
        ],
    },
    {
        "id": "agentic-evans-domain-driven-design",
        "book": "Domain-Driven Design: Tackling Complexity in the Heart of Software - Eric Evans",
        "desc": "Ubiquitous Language, Bounded Contexts, Entities, Value Objects, Aggregates, Repositories, Domain Services, and Anti-Corruption Layers.",
        "triggers": ["evans", "domain-driven-design", "ddd", "ubiquitous-language", "bounded-context", "aggregates", "anti-corruption-layer"],
        "foundations": [
            "Ubiquitous Language: A common, rigorous language shared by developers and domain experts, reflected directly in the source code.",
            "Bounded Context: A clear linguistic and architectural boundary within which a specific domain model applies and remains internally consistent.",
            "Aggregate Root Invariant: Aggregates are clusters of associated objects treated as a unit for data changes; all external access must go through the Root.",
        ],
        "protocol": "Model software domains around strict Aggregate boundaries. Enforce Ubiquitous Language consistently in types, variable names, and database schemas.",
        "anti_patterns": [
            "Anemic domain models where entities are dumb data bags manipulated by bloated procedural services.",
            "Leaking model concepts across Bounded Contexts without an Anti-Corruption Layer.",
        ],
    },
    {
        "id": "agentic-meadows-systems-thinking-feedback",
        "book": "Thinking in Systems: A Primer - Donella H. Meadows",
        "desc": "Stocks and flows, feedback loops (balancing and reinforcing), delays, system archetypes, and the 12 leverage points to intervene in a system.",
        "triggers": ["meadows", "systems-thinking", "stocks-and-flows", "feedback-loops", "leverage-points", "system-archetypes", "delay-dynamics"],
        "foundations": [
            "Stock and Flow Dynamics: Stock is the memory of history (codebase size, tech debt, bug count); flow is the rate of change (commit rate, fix rate).",
            "Balancing vs Reinforcing Loops: Reinforcing loops generate exponential growth or collapse; balancing loops resist change and enforce equilibrium.",
            "High-Leverage Interventions: Changing goals, paradigms, and system rules produces orders of magnitude more impact than tweaking numerical parameters.",
        ],
        "protocol": "Analyze software architectures as living dynamic systems. Target high-leverage intervention points (e.g. automated CI gates, compiler types) rather than surface symptoms.",
        "anti_patterns": [
            "Treating systemic bugs as isolated one-off errors without addressing the reinforcing feedback loops that cause them.",
            "Ignoring delays in feedback loops, causing over-correction and catastrophic oscillations in codebase refactoring.",
        ],
    },
    {
        "id": "agentic-kelly-autonomous-cognition-flows",
        "book": "The Inevitable: Understanding the 12 Technological Forces That Will Shape Our Future - Kevin Kelly",
        "desc": "Becoming, cognifying, flowing, screening, accessing, sharing, filtering, remixing, tracking, questioning, beginning, and autonomous intelligence flows.",
        "triggers": ["kelly", "kevin-kelly", "the-inevitable", "cognifying", "autonomous-flows", "remixing", "technological-forces", "future-ai-systems"],
        "foundations": [
            "Cognifying Everything: Infusing cheap, ubiquitous autonomous intelligence into every tool, compiler, and developer interface.",
            "Perpetual 'Becoming': All software is permanently in beta; continuous upgrades and dynamic mutations replace static finished releases.",
            "Remixing & Flowing: Value shifts from static copyrighted code repositories to real-time, dynamic streams of composable agent skills.",
        ],
        "protocol": "Design agentic tools as continuous, cognified streams. Facilitate effortless remixing and composition of specialized agent capabilities across project tasks.",
        "anti_patterns": [
            "Assuming software reaches a 'finished' static state, neglecting continuous self-updating adaptability.",
            "Building rigid monolithic tools that cannot be remixed into larger agentic workflows.",
        ],
    },
]

def generate_markdown(skill):
    """Generate the full content of SKILL.md for a given skill definition."""
    triggers_str = ", ".join(f'"{t}"' for t in skill["triggers"])
    
    foundations_md = "\n".join(f"{i+1}. **{f}**" for i, f in enumerate(skill["foundations"]))
    anti_patterns_md = "\n".join(f"- **{ap}**" for ap in skill["anti_patterns"])
    
    content = f"""---
name: {skill["id"]}
description: "{skill["desc"]}"
triggers: [{triggers_str}]
---

# {skill["id"]}
> Based on **{skill["book"]}**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

{foundations_md}

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
{skill["protocol"]}

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

{anti_patterns_md}

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "{skill["triggers"][0]}"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
"""
    return content

def main():
    print(f"Generating all {len(SKILLS)} Agentic Engineering skill packages...")
    created_count = 0
    
    for skill in SKILLS:
        skill_dir = SKILLS_DIR / skill["id"]
        skill_dir.mkdir(parents=True, exist_ok=True)
        skill_file = skill_dir / "SKILL.md"
        
        content = generate_markdown(skill)
        skill_file.write_text(content, encoding="utf-8")
        created_count += 1

    print(f"[+] Successfully generated {created_count} SKILL.md packages in {SKILLS_DIR}")

if __name__ == "__main__":
    main()
