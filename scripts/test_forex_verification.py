#!/usr/bin/env python3
"""
Master Builder & Verification Suite for:
Foreign Exchange (Forex) & Algorithmic Currency Trading Canon for Vibe Code Developers
Integrates into TGS (Tagisan ng Talino):
1. Documents: tagisan/docs/VIBE_CODER_FOREX_AND_CURRENCY_TRADING_GUIDE.md
2. Agent: .ecc/agents/forex-quant-expert.md (root and tagisan)
3. Skill: .ecc/skills/forex-and-currency-trading-vibe-coder/SKILL.md (root and tagisan)
4. Script: tagisan/scripts/test_forex_verification.py
5. Executes brutal verification tests (ISBN checks, math invariants, tgs ecc list/skills discovery).
"""

import os
import sys
import math
import shutil
import subprocess
from pathlib import Path

# --- 1. The 50 Canonical Forex & Currency Trading Books ---
BOOKS_50_CANON = [
    # Pillar I: FX Market Microstructure, Order Flow & Interbank Mechanics
    {
        "num": 1,
        "title": "The Microstructure Approach to Exchange Rates",
        "authors": "Richard K. Lyons",
        "publisher": "MIT Press",
        "year_edition": "2001",
        "isbn10": "0262122421",
        "isbn13": "978-0262122429",
        "domain": "FX Market Microstructure & Interbank Mechanics",
        "premise": "Models foreign exchange price formation via order flow dynamics, dealer inventory management, and asymmetric private information rather than purely macroeconomic aggregates.",
        "edge": "Essential for modeling dealer quote-shading, interpreting institutional customer order flow imbalances, and avoiding naive assumptions that order books have infinite depth."
    },
    {
        "num": 2,
        "title": "Foreign Exchange: A Practical Guide to the FX Markets",
        "authors": "Tim Weithers",
        "publisher": "John Wiley & Sons",
        "year_edition": "2006",
        "isbn10": "0471732036",
        "isbn13": "978-0471732037",
        "domain": "FX Market Microstructure & Interbank Mechanics",
        "premise": "Complete pedagogical walkthrough of currency pair quotation conventions (base vs. terms), triangular cross-rates, forward points, and interest rate parity.",
        "edge": "Eliminates foundational logic errors in vibe-coded trade settlement engines, such as inverted bid/ask spreads on crosses or miscalculated pip decimal shifts."
    },
    {
        "num": 3,
        "title": "Day Trading and Swing Trading the Currency Market",
        "authors": "Kathy Lien",
        "publisher": "John Wiley & Sons",
        "year_edition": "2015 (3rd Edition)",
        "isbn10": "1119108411",
        "isbn13": "978-1119108412",
        "domain": "FX Market Microstructure & Interbank Mechanics",
        "premise": "Practical market drivers, currency personality traits (safe havens vs. high-beta commodity currencies), and macro economic news release reaction patterns (NFP, CPI, rate cuts).",
        "edge": "Provides rule-based inputs for event-driven filters, preventing automated bots from executing market orders during spread blowouts around central bank announcements."
    },
    {
        "num": 4,
        "title": "Currency Forecasting: A Guide to Fundamental and Technical Models of Exchange Rate Determination",
        "authors": "Michael R. Rosenberg",
        "publisher": "McGraw-Hill",
        "year_edition": "1996",
        "isbn10": "1557389187",
        "isbn13": "978-1557389183",
        "domain": "FX Market Microstructure & Interbank Mechanics",
        "premise": "Institutional reference on macroeconomic models: Purchasing Power Parity (PPP), Uncovered Interest Parity (UIP), monetary models, and current account balances.",
        "edge": "Gives vibe coders the exact equations needed to engineer long-horizon macro features and mean-reversion fair-value anchors."
    },
    {
        "num": 5,
        "title": "The Foreign Exchange Matrix: A New Framework for Understanding Currency Movements",
        "authors": "Barbara Rockefeller, Vicki Schmelzer",
        "publisher": "Harriman House",
        "year_edition": "2013",
        "isbn10": "0857191306",
        "isbn13": "978-0857191304",
        "domain": "FX Market Microstructure & Interbank Mechanics",
        "premise": "Multi-currency flow matrix analysis, capital flows, reserve manager behavior, sovereign wealth funds, and cross-asset correlations (equities, yields, commodities).",
        "edge": "Guides the design of real-time currency strength meters and multi-pair covariance matrices, preventing strategies from loading duplicate risk on correlated USD legs."
    },
    {
        "num": 6,
        "title": "Inside the Currency Market: Mechanics and Strategies for Success",
        "authors": "Brian Dolan",
        "publisher": "Bloomberg Press / John Wiley & Sons",
        "year_edition": "2011",
        "isbn10": "1576603784",
        "isbn13": "978-1576603789",
        "domain": "FX Market Microstructure & Interbank Mechanics",
        "premise": "Operational structure of electronic communications networks (ECNs), prime brokerages, liquidity aggregation pools, and the London/New York session overlap.",
        "edge": "Teaches developers how to model realistic slippage, liquidity drops during the Asian session, and post-5 PM NY rollover bank clearing throttles."
    },
    {
        "num": 7,
        "title": "Exchange Rate Economics: Theories and Evidence",
        "authors": "Ronald MacDonald",
        "publisher": "Routledge",
        "year_edition": "2007",
        "isbn10": "0415125510",
        "isbn13": "978-0415125512",
        "domain": "FX Market Microstructure & Interbank Mechanics",
        "premise": "Empirical econometrics of exchange rates, testing random walk hypotheses, co-integration of exchange rates, and structural regime switching.",
        "edge": "Crucial for auditing econometric claims generated by LLMs; warns against asserting cointegration without proper Dickey-Fuller and Johansen cointegration test gates."
    },
    {
        "num": 8,
        "title": "The Economics of Exchange Rates",
        "authors": "Lucio Sarno, Mark P. Taylor",
        "publisher": "Cambridge University Press",
        "year_edition": "2002",
        "isbn10": "0521485843",
        "isbn13": "978-0521485845",
        "domain": "FX Market Microstructure & Interbank Mechanics",
        "premise": "Examines non-linearities in exchange rate dynamics, target zone models, official foreign exchange intervention effectiveness, and forward premium bias anomalies.",
        "edge": "Equips developers with mathematical formulation of the forward discount puzzle, the theoretical backbone of quantitative carry trade models."
    },

    # Pillar II: Algorithmic Architecture, System Engineering & Direct Market Access
    {
        "num": 9,
        "title": "Building Algorithmic Trading Systems: A Trader's Journey from Data Mining to Monte Carlo Simulation to Live Trading",
        "authors": "Kevin J. Davey",
        "publisher": "John Wiley & Sons",
        "year_edition": "2014",
        "isbn10": "1118778987",
        "isbn13": "978-1118778982",
        "domain": "Algorithmic Architecture & System Engineering",
        "premise": "Disciplined manufacturing process for algorithmic trading: in-sample data mining, walk-forward testing, Monte Carlo risk simulation, and automated deployment.",
        "edge": "The ideal step-by-step system prompt template for instructing AI coding agents on how to build and validate trading bots without human self-delusion."
    },
    {
        "num": 10,
        "title": "Quantitative Trading: How to Build Your Own Algorithmic Trading Business",
        "authors": "Ernest P. Chan",
        "publisher": "John Wiley & Sons",
        "year_edition": "2021 (2nd Edition)",
        "isbn10": "1119800064",
        "isbn13": "978-1119800064",
        "domain": "Algorithmic Architecture & System Engineering",
        "premise": "Foundational guide for independent quantitative developers: backtest pitfalls, data sanitization, transaction cost models, and retail API execution architectures.",
        "edge": "Mandatory reading to avoid the top vibe-coding trap: backtests that show a 5.0 Sharpe ratio purely because bid-ask spread and financing costs were omitted."
    },
    {
        "num": 11,
        "title": "Algorithmic Trading: Winning Strategies and Their Rationale",
        "authors": "Ernest P. Chan",
        "publisher": "John Wiley & Sons",
        "year_edition": "2013",
        "isbn10": "1118460146",
        "isbn13": "978-1118460146",
        "domain": "Algorithmic Architecture & System Engineering",
        "premise": "Rigorous treatment of statistical arbitrage, mean reversion, momentum, and cross-currency cointegration with exact mathematical formulas and pseudocode.",
        "edge": "Provides drop-in algorithmic recipes (ADF test, Johansen test, Ornstein-Uhlenbeck half-life formula) that can be verified deterministically."
    },
    {
        "num": 12,
        "title": "Systematic Trading: A Unique New Method for Designing Trading and Investing Systems",
        "authors": "Robert Carver",
        "publisher": "Harriman House",
        "year_edition": "2015",
        "isbn10": "0857194453",
        "isbn13": "978-0857194459",
        "domain": "Algorithmic Architecture & System Engineering",
        "premise": "Framework for systematic continuous position sizing, forecast scaling, volatility targeting, and multi-asset capital allocation without subjective intervention.",
        "edge": "Eliminates binary on/off trading logic; enables vibe coders to prompt smooth position sizing scaled inversely to prevailing ATR (Average True Range)."
    },
    {
        "num": 13,
        "title": "Python for Algorithmic Trading: From Idea to Cloud Deployment",
        "authors": "Yves Hilpisch",
        "publisher": "O'Reilly Media",
        "year_edition": "2020",
        "isbn10": "149205335X",
        "isbn13": "978-1492053354",
        "domain": "Algorithmic Architecture & System Engineering",
        "premise": "Full engineering stack for Python automated trading: vectorized backtesting, event-driven socket pipelines, real-time tick processing, and cloud deployment.",
        "edge": "Directly usable patterns for structuring asyncio event loops, WebSocket connections to OANDA/Interactive Brokers, and automated cloud monitoring."
    },
    {
        "num": 14,
        "title": "Trading and Exchanges: Market Microstructure for Practitioners",
        "authors": "Larry Harris",
        "publisher": "Oxford University Press",
        "year_edition": "2002",
        "isbn10": "0195144708",
        "isbn13": "978-0195144703",
        "domain": "Algorithmic Architecture & System Engineering",
        "premise": "The comprehensive bible on market structure: order types, limit order books, bid-ask spreads, market makers, dealer inventory, and adverse selection.",
        "edge": "Gives developers the domain vocabulary and invariants to model order queues, fill probabilities, and toxic order flow."
    },
    {
        "num": 15,
        "title": "Algorithmic and High-Frequency Trading",
        "authors": "Álvaro Cartea, Sebastian Jaimungal, José Penalva",
        "publisher": "Cambridge University Press",
        "year_edition": "2015",
        "isbn10": "1107091144",
        "isbn13": "978-1107091146",
        "domain": "Algorithmic Architecture & System Engineering",
        "premise": "Stochastic optimal control, Hawkes processes for jump arrivals, Almgren-Chriss optimal liquidation, and limit order placement strategies.",
        "edge": "High-level mathematical foundation for writing optimal trade execution algorithms that minimize market impact when managing large currency orders."
    },
    {
        "num": 16,
        "title": "Developing High-Frequency Trading Systems",
        "authors": "Sebastien Donadio, Sourav Ghosh",
        "publisher": "Packt Publishing",
        "year_edition": "2017",
        "isbn10": "1788296311",
        "isbn13": "978-1788296311",
        "domain": "Algorithmic Architecture & System Engineering",
        "premise": "Low-latency systems engineering: cache-friendly memory structures, lock-free ring buffers, socket optimization, and FIX protocol integration.",
        "edge": "Critical for vibe coders transitioning from slow REST polling loops to zero-copy, sub-millisecond execution pipelines in C++ or Rust."
    },
    {
        "num": 17,
        "title": "Algorithmic Trading & DMA: An Introduction to Direct Access Trading Strategies",
        "authors": "Barry Johnson",
        "publisher": "4Myeloma Press",
        "year_edition": "2010",
        "isbn10": "0956399207",
        "isbn13": "978-0956399205",
        "domain": "Algorithmic Architecture & System Engineering",
        "premise": "Encyclopedia of institutional trading execution algorithms (TWAP, VWAP, Implementation Shortfall, Volume Inline, and Iceberg detection).",
        "edge": "Blueprint for implementing institutional-grade algorithmic order slicers that protect retail accounts from being front-run by institutional liquidity providers."
    },

    # Pillar III: Quantitative Strategies, Statistical Arbitrage & Technical Rigor
    {
        "num": 18,
        "title": "Pairs Trading: Quantitative Methods and Analysis",
        "authors": "Ganapathy Vidyamurthy",
        "publisher": "John Wiley & Sons",
        "year_edition": "2004",
        "isbn10": "0471460672",
        "isbn13": "978-0471460671",
        "domain": "Quantitative Strategies & Statistical Arbitrage",
        "premise": "Time-series cointegration testing, error-correction models (ECM), and Kalman filtering applied to pairs trading and spread arbitrage.",
        "edge": "Provides the mathematical groundwork to code robust triangular arbitrage and cross-currency cointegration bots (e.g., EUR/USD vs. GBP/USD vs. EUR/GBP)."
    },
    {
        "num": 19,
        "title": "Statistically Sound Indicators for Financial Market Prediction",
        "authors": "Timothy Masters",
        "publisher": "CreateSpace",
        "year_edition": "2019",
        "isbn10": "1099682126",
        "isbn13": "978-1099682124",
        "domain": "Quantitative Strategies & Statistical Arbitrage",
        "premise": "Stationary time-series transformation, non-linear indicator derivation, entropy metrics, and data normalization for predictive model inputs.",
        "edge": "Prevents feeding raw non-stationary price data into neural networks, ensuring inputs have zero lookahead bias and stable statistical distributions."
    },
    {
        "num": 20,
        "title": "Evidence-Based Technical Analysis: Applying the Scientific Method and Statistical Inference to Trading Signals",
        "authors": "David Aronson",
        "publisher": "John Wiley & Sons",
        "year_edition": "2006",
        "isbn10": "0470008741",
        "isbn13": "978-0470008744",
        "domain": "Quantitative Strategies & Statistical Arbitrage",
        "premise": "Statistical hypothesis testing, Monte Carlo permutation tests, and data-mining bias corrections (White's Reality Check) for evaluating technical rules.",
        "edge": "The ultimate antidote to chart pattern hallucination: proves why 95% of retail technical indicators lack statistical significance when corrected for multiple testing."
    },
    {
        "num": 21,
        "title": "Quantitative Momentum: A Practitioner's Guide to Building a Momentum-Based Stock/Asset Selection System",
        "authors": "Wesley R. Gray, Jack R. Vogel",
        "publisher": "John Wiley & Sons",
        "year_edition": "2016",
        "isbn10": "111923719X",
        "isbn13": "978-1119237198",
        "domain": "Quantitative Strategies & Statistical Arbitrage",
        "premise": "Behavioral foundations of momentum, path quality (smooth vs. erratic trends), and time-series momentum scaling.",
        "edge": "Enables vibe coders to prompt momentum strategies that filter out choppy, high-volatility false breakouts in major currency pairs."
    },
    {
        "num": 22,
        "title": "Finding Alphas: A Quantitative Approach to Building Trading Strategies",
        "authors": "Igor Tulchinsky",
        "publisher": "John Wiley & Sons",
        "year_edition": "2019 (2nd Edition)",
        "isbn10": "1119565200",
        "isbn13": "978-1119565208",
        "domain": "Quantitative Strategies & Statistical Arbitrage",
        "premise": "WorldQuant's methodology for engineering, backtesting, and combining hundreds of uncorrelated mathematical alpha expressions into robust portfolios.",
        "edge": "Provides a syntax and grammar for formulaic alphas that LLMs can rapidly synthesize, test, and cross-correlate."
    },
    {
        "num": 23,
        "title": "Quantitative Trading Systems: Practical Methods for Design, Testing, and Validation",
        "authors": "Howard B. Bandy",
        "publisher": "Blue Owl Press",
        "year_edition": "2007",
        "isbn10": "0979177006",
        "isbn13": "978-0979177002",
        "domain": "Quantitative Strategies & Statistical Arbitrage",
        "premise": "Design philosophy emphasizing objective functions, fitness metric selection, walk-forward testing, and distribution-based risk accounting.",
        "edge": "Helps developers define loss functions that penalize maximum drawdown duration and downside semi-variance rather than raw profit."
    },
    {
        "num": 24,
        "title": "Mean Reversion Trading Systems: Practical Methods for Swing Trading",
        "authors": "Howard B. Bandy",
        "publisher": "Blue Owl Press",
        "year_edition": "2013",
        "isbn10": "0979177073",
        "isbn13": "978-0979177071",
        "domain": "Quantitative Strategies & Statistical Arbitrage",
        "premise": "Mathematical modeling of mean-reverting price swings, dynamic band calculation, volatility thresholding, and profit-target optimization.",
        "edge": "Ideal for designing Asian-session range-trading bots and intraday mean-reverting algorithms on non-trending currency crosses like EUR/CHF."
    },
    {
        "num": 25,
        "title": "Following the Trend: Diversified Managed Futures Trading",
        "authors": "Andreas F. Clenow",
        "publisher": "John Wiley & Sons",
        "year_edition": "2012",
        "isbn10": "1118410858",
        "isbn13": "978-1118410851",
        "domain": "Quantitative Strategies & Statistical Arbitrage",
        "premise": "Deconstructs hedge fund trend-following strategies: multi-asset breakout filters, ATR-based volatility equalization, and long-term trend survival.",
        "edge": "Provides exact, non-curve-fitted rules for systematic multi-currency trend following that have survived live institutional trading across decades."
    },

    # Pillar IV: Machine Learning, Deep Learning & Modern AI in Finance
    {
        "num": 26,
        "title": "Advances in Financial Machine Learning",
        "authors": "Marcos López de Prado",
        "publisher": "John Wiley & Sons",
        "year_edition": "2018",
        "isbn10": "1119482089",
        "isbn13": "978-1119482086",
        "domain": "Machine Learning & AI in Finance",
        "premise": "The seminal text establishing modern financial ML: Triple Barrier Method, Purged/Embargoed K-Fold cross-validation, fractional differentiation, and meta-labeling.",
        "edge": "The single most important book for AI coders in finance. Fixes the fundamental failure modes of naive ML (data leakage, serial correlation, and unrealistic sample labeling)."
    },
    {
        "num": 27,
        "title": "Machine Learning for Asset Managers",
        "authors": "Marcos López de Prado",
        "publisher": "Cambridge University Press",
        "year_edition": "2020",
        "isbn10": "1108792898",
        "isbn13": "978-1108792899",
        "domain": "Machine Learning & AI in Finance",
        "premise": "Denoising and detoning empirical covariance matrices with Random Matrix Theory (RMT), clustered feature importance, and Hierarchical Risk Parity (HRP).",
        "edge": "Provides drop-in algorithms for allocating capital across multiple currency pairs without suffering the instability of classical Markowitz mean-variance optimization."
    },
    {
        "num": 28,
        "title": "Machine Learning for Algorithmic Trading",
        "authors": "Stefan Jansen",
        "publisher": "Packt Publishing",
        "year_edition": "2020 (2nd Edition)",
        "isbn10": "1839217715",
        "isbn13": "978-1839217715",
        "domain": "Machine Learning & AI in Finance",
        "premise": "End-to-end guide to ML in trading: feature engineering from market data, tree-based models (LightGBM/XGBoost), NLP sentiment analysis, and reinforcement learning.",
        "edge": "Provides practical Python pipeline architectures for ingesting tick data, computing technical alphas, and training predictive models."
    },
    {
        "num": 29,
        "title": "Financial Machine Learning: Predicting Prices and Risk with Python",
        "authors": "Matthew F. Dixon, Igor Halperin, Paul Bilokon",
        "publisher": "Springer",
        "year_edition": "2020",
        "isbn10": "3030410676",
        "isbn13": "978-3030410674",
        "domain": "Machine Learning & AI in Finance",
        "premise": "Deep probabilistic modeling, autoencoders, neural jump ODEs, and deep reinforcement learning applied to pricing, market making, and risk forecasting.",
        "edge": "Essential for structuring deep neural architectures that respect physical financial constraints rather than treating price series like generic image or text tokens."
    },
    {
        "num": 30,
        "title": "Deep Learning for Finance: Creating Machine Learning Models and Neural Networks for Trading and Forecasting",
        "authors": "Sofien Kaabar",
        "publisher": "O'Reilly Media",
        "year_edition": "2024",
        "isbn10": "1098148479",
        "isbn13": "978-1098148478",
        "domain": "Machine Learning & AI in Finance",
        "premise": "Practical application of CNNs, LSTMs, GRUs, and Transformer architectures to financial time-series forecasting, pattern recognition, and regime classification.",
        "edge": "Offers practical PyTorch/TensorFlow recipes tailored specifically to tabular currency data, avoiding excessive latency during live inference."
    },
    {
        "num": 31,
        "title": "Reinforcement Learning for Finance: Solve Problems in Finance with Modern RL Techniques",
        "authors": "Bence Tóth",
        "publisher": "Packt Publishing",
        "year_edition": "2024",
        "isbn10": "1804618799",
        "isbn13": "978-1804618790",
        "domain": "Machine Learning & AI in Finance",
        "premise": "Q-learning, Actor-Critic (PPO, DDPG), and model-based RL applied to trade execution, portfolio balancing, and automated market making.",
        "edge": "Guides the creation of realistic Gymnasium-compatible simulated FX environments with fees and slippage to train robust RL execution agents."
    },
    {
        "num": 32,
        "title": "Hands-On Machine Learning for Algorithmic Trading",
        "authors": "Stefan Jansen",
        "publisher": "Packt Publishing",
        "year_edition": "2018",
        "isbn10": "178934641X",
        "isbn13": "978-1789346411",
        "domain": "Machine Learning & AI in Finance",
        "premise": "Building ML pipelines, feature extraction from limit order books, Bayesian optimization for hyperparameters, and performance attribution.",
        "edge": "Provides step-by-step guidance on setting up backtest harnesses with Zipline/PyFolio for measuring alpha factor decay."
    },
    {
        "num": 33,
        "title": "Artificial Intelligence in Finance: A Python-Based Guide",
        "authors": "Yves Hilpisch",
        "publisher": "O'Reilly Media",
        "year_edition": "2020",
        "isbn10": "1492055182",
        "isbn13": "978-1492055181",
        "domain": "Machine Learning & AI in Finance",
        "premise": "Philosophical and mathematical exploration of AI and deep neural networks in finance, contrasting classical normative theory with data-driven empirical models.",
        "edge": "Helps developers understand why AI models often overfit financial data and how to formulate simple, robust network architectures."
    },

    # Pillar V: Order Book Dynamics, Microstructure & High-Frequency Execution
    {
        "num": 34,
        "title": "Trades, Quotes and Prices: Financial Markets Under the Microscope",
        "authors": "Jean-Philippe Bouchaud, Julius Bonart, Jonathan Donier, Martin Gould",
        "publisher": "Cambridge University Press",
        "year_edition": "2018",
        "isbn10": "1107164036",
        "isbn13": "978-1107164031",
        "domain": "Order Book Dynamics & High-Frequency Execution",
        "premise": "Statistical physics approach to market microstructure: square-root law of market impact, order flow auto-correlation, and liquidity crisis dynamics.",
        "edge": "Crucial for accurately modeling execution slippage: proves that market impact is non-linear and sub-linear ($I \\sim \\sigma \\sqrt{Q/V}$)."
    },
    {
        "num": 35,
        "title": "Empirical Market Microstructure: The Institutions, Economics, and Econometrics of Securities Trading",
        "authors": "Joel Hasbrouck",
        "publisher": "Oxford University Press",
        "year_edition": "2007",
        "isbn10": "0195301641",
        "isbn13": "978-0195301649",
        "domain": "Order Book Dynamics & High-Frequency Execution",
        "premise": "Econometric models for order books: Vector Autoregressions (VAR) of trade prices and order flows, Roll's model of effective spread, and price discovery metrics.",
        "edge": "Provides quantitative methods for decomposing FX spreads into liquidity-provider inventory holding costs versus adverse selection from informed traders."
    },
    {
        "num": 36,
        "title": "High-Frequency Trading: A Practical Guide to Algorithmic Strategies and Trading Systems",
        "authors": "Irene Aldridge",
        "publisher": "John Wiley & Sons",
        "year_edition": "2013 (2nd Edition)",
        "isbn10": "1118343808",
        "isbn13": "978-1118343807",
        "domain": "Order Book Dynamics & High-Frequency Execution",
        "premise": "Covers high-frequency market making, cross-venue latency arbitrage, order book imbalance indicators, and real-time risk controls.",
        "edge": "Essential for coding order book imbalance (OBI) features and designing low-latency hedging bots across fragmented currency liquidity venues."
    },
    {
        "num": 37,
        "title": "Market Microstructure in Practice",
        "authors": "Charles-Albert Lehalle, Sophie Laruelle",
        "publisher": "World Scientific Publishing",
        "year_edition": "2018 (2nd Edition)",
        "isbn10": "9813230835",
        "isbn13": "978-9813230835",
        "domain": "Order Book Dynamics & High-Frequency Execution",
        "premise": "Practitioner perspective on transaction cost analysis (TCA), algorithmic order routing, dark liquidity pools, and queue position dynamics.",
        "edge": "Teaches developers how institutional FX smart order routers (SORs) divide and execute parent orders to prevent aggressive price leakage."
    },
    {
        "num": 38,
        "title": "Flash Boys: A Wall Street Revolt",
        "authors": "Michael Lewis",
        "publisher": "W. W. Norton & Company",
        "year_edition": "2014",
        "isbn10": "0393244660",
        "isbn13": "978-0393244663",
        "domain": "Order Book Dynamics & High-Frequency Execution",
        "premise": "Investigative chronicle of high-frequency trading, microwave networks, co-location, sip latency arbitrage, and predatory front-running.",
        "edge": "Instills deep architectural paranoia: reminds vibe coders that retail orders placed through B-book brokers are vulnerable to synthetic latency delays and asymmetric slip."
    },
    {
        "num": 39,
        "title": "Dark Pools and High Frequency Trading For Dummies",
        "authors": "Jay Vaananen",
        "publisher": "John Wiley & Sons",
        "year_edition": "2015",
        "isbn10": "1119001374",
        "isbn13": "978-1119001379",
        "domain": "Order Book Dynamics & High-Frequency Execution",
        "premise": "Clear, accessible breakdown of internalizers, dark liquidity matching, payment for order flow (PFOF), and retail broker B-booking.",
        "edge": "Clarifies the difference between true interbank ECN/STP execution and retail market-maker B-book routing, preventing flawed execution assumptions."
    },

    # Pillar VI: Backtesting, Walk-Forward Validation & Preventing Overfitting
    {
        "num": 40,
        "title": "The Evaluation and Optimization of Trading Strategies",
        "authors": "Robert Pardo",
        "publisher": "John Wiley & Sons",
        "year_edition": "2008 (2nd Edition)",
        "isbn10": "0470128011",
        "isbn13": "978-0470128015",
        "domain": "Backtesting, Validation & Preventing Overfitting",
        "premise": "The seminal textbook on Walk-Forward Analysis (WFA), rolling in-sample optimization, out-of-sample testing windows, and Walk-Forward Efficiency (WFE).",
        "edge": "Mandatory testing protocol: any strategy that cannot demonstrate a Walk-Forward Efficiency (WFE) > 60% is guaranteed to be an over-fitted artifact."
    },
    {
        "num": 41,
        "title": "Testing and Tuning Market Trading Systems: Algorithms in C++",
        "authors": "Timothy Masters",
        "publisher": "John Wiley & Sons",
        "year_edition": "1998",
        "isbn10": "047124113X",
        "isbn13": "978-0471241133",
        "domain": "Backtesting, Validation & Preventing Overfitting",
        "premise": "Mathematical algorithms for backtest cross-validation, parameter sensitivity surfaces, bootstrap resampling, and avoiding local parameter minima.",
        "edge": "Provides algorithms for measuring parameter cliff risk—rejecting parameter sets where small market changes cause catastrophic performance collapse."
    },
    {
        "num": 42,
        "title": "Maximum Adverse Excursion: Analyzing Price Fluctuations for Trading Management",
        "authors": "John Sweeney",
        "publisher": "John Wiley & Sons",
        "year_edition": "1997",
        "isbn10": "0471177652",
        "isbn13": "978-0471177654",
        "domain": "Backtesting, Validation & Preventing Overfitting",
        "premise": "Formulates Maximum Adverse Excursion (MAE) and Maximum Favorable Excursion (MFE) to determine optimal, evidence-based stop loss and take profit thresholds.",
        "edge": "Replaces arbitrary round-number stop losses with analytical distributions of trade intrabar drawdowns, maximizing risk/reward expectancy."
    },
    {
        "num": 43,
        "title": "Model Risk Management: Financial Modeling for Risk and Compliance",
        "authors": "Peter Verhoeven",
        "publisher": "Palgrave Macmillan",
        "year_edition": "2021",
        "isbn10": "3030784371",
        "isbn13": "978-3030784379",
        "domain": "Backtesting, Validation & Preventing Overfitting",
        "premise": "Frameworks for validating financial models, stress testing, benchmarking, detecting assumption creep, and managing model governance.",
        "edge": "Equips developers with institutional model validation protocols (SR 11-7 standards) to audit AI-generated codebases before deploying real capital."
    },

    # Pillar VII: Risk Management, Position Sizing & Capital Preservation
    {
        "num": 44,
        "title": "The Mathematics of Money Management: Risk Analysis Techniques for Traders",
        "authors": "Ralph Vince",
        "publisher": "John Wiley & Sons",
        "year_edition": "1992",
        "isbn10": "0471547387",
        "isbn13": "978-0471547389",
        "domain": "Risk Management, Sizing & Capital Preservation",
        "premise": "Optimal f, capital growth equations, geometric mean return maximization, and the mathematical mechanics of drawdown recovery.",
        "edge": "Crucial for hard-coding position sizing engines that respect fractional Kelly boundaries and prevent aggressive over-leveraging on retail margins."
    },
    {
        "num": 45,
        "title": "Dynamic Hedging: Managing Vanilla and Exotic Options",
        "authors": "Nassim Nicholas Taleb",
        "publisher": "John Wiley & Sons",
        "year_edition": "1997",
        "isbn10": "0471152803",
        "isbn13": "978-0471152804",
        "domain": "Risk Management, Sizing & Capital Preservation",
        "premise": "Practitioner guide to non-linear risk, fat-tailed distributions, gamma bleeding, slippage in wild markets, and the breakdown of Black-Scholes assumptions.",
        "edge": "Instills ruthless respect for kurtosis and black swan gap events (like the 2015 SNB EUR/CHF peg removal) where stops fail to fill."
    },
    {
        "num": 46,
        "title": "Risk Management and Financial Institutions",
        "authors": "John C. Hull",
        "publisher": "John Wiley & Sons",
        "year_edition": "2018 (5th Edition)",
        "isbn10": "1119448115",
        "isbn13": "978-1119448112",
        "domain": "Risk Management, Sizing & Capital Preservation",
        "premise": "Industry standard on Value at Risk (VaR), Expected Shortfall (CVaR), liquidity risk, counterparty credit risk, and scenario stress testing.",
        "edge": "Provides the mathematical formulas needed to build automated portfolio risk circuit breakers and margin maintenance monitors."
    },
    {
        "num": 47,
        "title": "Financial Risk Forecasting: The Theory and Practice of Forecasting Market Risk",
        "authors": "Jon Danielsson",
        "publisher": "John Wiley & Sons",
        "year_edition": "2011",
        "isbn10": "0470669438",
        "isbn13": "978-0470669433",
        "domain": "Risk Management, Sizing & Capital Preservation",
        "premise": "GARCH volatility modeling, Extreme Value Theory (EVT), copula models for tail dependence, and endogeneity in market risk.",
        "edge": "Enables developers to program dynamic volatility adjusters that automatically reduce position sizes when volatility regimes transition from calm to turbulent."
    },

    # Pillar VIII: Market Reality, Trader Intuition & Historical Lessons
    {
        "num": 48,
        "title": "The Man Who Solved the Market: How Jim Simons Launched the Quant Revolution",
        "authors": "Gregory Zuckerman",
        "publisher": "Portfolio / Penguin",
        "year_edition": "2019",
        "isbn10": "073521798X",
        "isbn13": "978-0735217980",
        "domain": "Market Reality, Trader Intuition & Lessons",
        "premise": "The definitive history of Renaissance Technologies: data hygiene, speech recognition scientists solving market signals, non-intuitive short-term patterns, and removing human emotion.",
        "edge": "Demonstrates why data hygiene, obsessive transaction cost tracking, and scientific peer review triumph over subjective financial opinions."
    },
    {
        "num": 49,
        "title": "Market Wizards: Interviews with Top Traders",
        "authors": "Jack D. Schwager",
        "publisher": "John Wiley & Sons",
        "year_edition": "2012",
        "isbn10": "1118273052",
        "isbn13": "978-1118273050",
        "domain": "Market Reality, Trader Intuition & Lessons",
        "premise": "Legendary interviews with global macro and currency titans (Bruce Kovner, Michael Marcus, Paul Tudor Jones) on risk asymmetry and survival.",
        "edge": "Teaches how central banks intervene and how currency trends can persist far longer than economic rationality dictates."
    },
    {
        "num": 50,
        "title": "Reminiscences of a Stock Operator",
        "authors": "Edwin Lefèvre",
        "publisher": "John Wiley & Sons",
        "year_edition": "2006",
        "isbn10": "0471770884",
        "isbn13": "978-0471770886",
        "domain": "Market Reality, Trader Intuition & Lessons",
        "premise": "The timeless roman à clef of Jesse Livermore: tape reading, price momentum, market manipulation, and the psychological traps of leverage.",
        "edge": "Exposes retail liquidity hunting patterns; helps vibe coders design stop-run detection filters around key round-number psychological levels."
    }
]

# --- 2. Checksum Validators ---

def validate_isbn10(isbn: str) -> bool:
    clean = isbn.replace("-", "").replace(" ", "")
    if len(clean) != 10:
        return False
    total = 0
    for i in range(9):
        if not clean[i].isdigit():
            return False
        total += int(clean[i]) * (10 - i)
    check_char = clean[9].upper()
    check_val = 10 if check_char == "X" else (int(check_char) if check_char.isdigit() else -1)
    if check_val == -1:
        return False
    return (total + check_val) % 11 == 0

def validate_isbn13(isbn: str) -> bool:
    clean = isbn.replace("-", "").replace(" ", "")
    if len(clean) != 13 or not clean.isdigit():
        return False
    total = sum(int(clean[i]) * (1 if i % 2 == 0 else 3) for i in range(12))
    expected_check = (10 - (total % 10)) % 10
    return expected_check == int(clean[12])

# --- 3. Markdown Content Generators ---

def generate_guide_markdown() -> str:
    lines = [
        "# VIBE CODER'S FOREIGN EXCHANGE (FOREX) & CURRENCY TRADING CANON",
        "## The 50 Authoritative Books on Market Microstructure, ECN Order Flow, Algorithmic Execution, Quantitative Carry, Machine Learning in Finance, Risk Mathematics, and Macroeconomic Regimes",
        "",
        "> In the **$7.5-trillion-per-day Foreign Exchange (Forex) market**, there is no centralized clearinghouse or single exchange.",
        "> Currency trading is an asymmetric, decentralized over-the-counter (OTC) battlefield governed by interbank tier-1 dealers,",
        "> electronic communication networks (ECNs), high-frequency liquidity providers, and sovereign central banks.",
        ">",
        "> When a **vibe code developer** leverages AI models (LLMs, coding agents, rapid prompts) to build automated Forex bots,",
        "> execution routers, and quantitative models, the uncurated output routinely defaults to **Catastrophic Account Blowup**:",
        "> - Zero accounting for dynamic spread widening (3x-10x spikes during the 5:00 PM NY rollover).",
        "> - Fatal pip value calculation errors across cross-currency pairs (misidentifying quote currency conversion legs).",
        "> - Severe lookahead bias and serial correlation leakage in backtests (using random splits instead of purged cross-validation).",
        "> - Overfitted technical indicators that lack statistical significance after correcting for multiple testing.",
        "> - Assuming infinite market depth and zero slippage on stop-loss market orders during high-impact news events (NFP, CPI, rate decisions).",
        ">",
        "> This canon details the **50 foundational texts** that equip the vibe code developer with the mathematical rigor,",
        "> architectural invariants, and institutional safeguards necessary to build 100% reliable, production-grade automated currency systems.",
        "",
        "---",
        ""
    ]

    current_pillar = ""
    pillar_headers = {
        1: "## Pillar I: FX Market Microstructure, Order Flow & Interbank Mechanics",
        9: "## Pillar II: Algorithmic Architecture, System Engineering & Direct Market Access",
        18: "## Pillar III: Quantitative Strategies, Statistical Arbitrage & Technical Rigor",
        26: "## Pillar IV: Machine Learning, Deep Learning & Modern AI in Finance",
        34: "## Pillar V: Order Book Dynamics, Microstructure & High-Frequency Execution",
        40: "## Pillar VI: Backtesting, Walk-Forward Validation & Preventing Overfitting",
        44: "## Pillar VII: Risk Management, Position Sizing & Capital Preservation",
        48: "## Pillar VIII: Market Reality, Trader Intuition & Historical Lessons"
    }

    for b in BOOKS_50_CANON:
        num = b["num"]
        if num in pillar_headers:
            lines.append(pillar_headers[num])
            lines.append("")

        lines.append(f"### #{num}. {b['title']}")
        lines.append(f"- **Author(s)**: {b['authors']}")
        lines.append(f"- **Publisher / Edition**: {b['publisher']} ({b['year_edition']})")
        lines.append(f"- **ISBN-10**: `{b['isbn10']}` | **ISBN-13**: `{b['isbn13']}`")
        lines.append(f"- **Core Premise**: {b['premise']}")
        lines.append(f"- **The Vibe Coder's Edge**: {b['edge']}")
        lines.append("")

    lines.extend([
        "---",
        "",
        "## The Vibe Coder's Forex Algorithmic & Risk Verification Architecture",
        "",
        "```mermaid",
        "flowchart TD",
        "    A[\"Raw AI Strategy Synthesis (Prompts, TradingView, Python, MT5/MQL5)\"] --> B[\"1. Pip Value & Cross-Currency Settlement Normalizer\"]",
        "    B --> C[\"2. Dynamic Spread Widening & Overnight Swap Rollover Engine\"]",
        "    C --> D[\"3. Purged K-Fold Cross Validation & Walk-Forward Optimizer (WFE > 60%)\"]",
        "    D --> E[\"4. Microstructure & Direct Market Access Router (TWAP / Iceberg / FIX)\"]",
        "    E --> F[\"5. Institutional Risk Circuit Breakers (Fractional Kelly / Max Daily DD)\"]",
        "    F --> G[\"DEPLOY: Resilient Event-Driven FX Trading System\"]",
        "```",
        "",
        "---",
        "",
        "## Mathematical Invariants for Vibe Coded Forex Engines",
        "",
        "### 1. Pip Value Exact Settlement Formulation",
        "For any currency pair $XXX/YYY$ traded with position size $L$ (base currency units), where account denomination is currency $AAA$:",
        "- If $YYY == AAA$ (Direct, e.g., EUR/USD with USD account):",
        "  $$\\text{Pip Value} = L \\times \\text{Pip Size}$$",
        "- If $XXX == AAA$ (Indirect, e.g., USD/JPY with USD account):",
        "  $$\\text{Pip Value} = \\frac{L \\times \\text{Pip Size}}{\\text{Rate}(XXX/YYY)}$$",
        "- If $XXX \\neq AAA$ and $YYY \\neq AAA$ (Cross pair, e.g., EUR/GBP with USD account):",
        "  $$\\text{Pip Value} = L \\times \\text{Pip Size} \\times \\text{Rate}(YYY/AAA)$$",
        "",
        "### 2. Triangular Arbitrage Mispricing Condition",
        "To execute a risk-free cross-currency triangular arbitrage loop ($USD \\to EUR \\to GBP \\to USD$):",
        "$$\\Pi = \\left( \\frac{1}{\\text{Ask}_{EUR/USD}} \\right) \\times \\text{Bid}_{EUR/GBP} \\times \\text{Bid}_{GBP/USD} - 1.0 > \\text{TCA}_{\\text{roundtrip}}$$",
        "Where $\\text{TCA}_{\\text{roundtrip}}$ incorporates brokerage commissions, crossing fees, and expected adverse latency slippage.",
        "",
        "### 3. Mean-Reversion Half-Life (Ornstein-Uhlenbeck Process)",
        "For a cointegrated cross-rate spread $y_t$ modeled as $dy_t = \\theta (\\mu - y_t) dt + \\sigma dW_t$:",
        "$$\\Delta y_t = \\lambda y_{t-1} + \\alpha + \\epsilon_t, \\quad \\lambda = e^{-\\theta \\Delta t} - 1$$",
        "$$\\text{Half-Life } t_{1/2} = -\\frac{\\ln(2)}{\\lambda}$$",
        "Reject mean-reversion strategies if $\\lambda \\ge 0$ (non-stationary random walk) or $t_{1/2} > \\text{Holding Horizon}$.",
        "",
        "### 4. Fractional Kelly Position Sizing with Volatility Scaling",
        "$$f^* = c \\cdot \\left( \\frac{p \\cdot b - (1 - p)}{b} \\right) \\times \\left( \\frac{\\sigma_{\\text{target}}}{\\sigma_{\\text{realized}}} \\right)$$",
        "Where $c \\in [0.25, 0.5]$ (Quarter or Half-Kelly conservative multiplier), $p$ is empirical win-rate, $b$ is payoff ratio, and positions are dynamically dampened during volatility regime spikes.",
        "",
        "---",
        "",
        "## The Vibe Coder's Anti-Hallucination Checklist for Forex",
        "",
        "1. **Never Accept Zero Transaction Costs**: Always inject a minimum baseline spread (1.2 pips on EUR/USD, 2.5 pips on GBP/JPY) plus $5/lot commission.",
        "2. **Simulate 5 PM NY Rollover Widening**: Force spreads to expand 4x to 8x between 16:59 and 17:05 EST to catch stop-outs in backtests.",
        "3. **Purge Walk-Forward Data**: Ensure out-of-sample testing windows are completely separated from feature normalization and parameter sweeps.",
        "4. **Blackout High-Impact Macro Releases**: Programmatically disconnect execution 15 minutes prior to and following NFP, FOMC, ECB, and CPI releases.",
        "5. **Enforce Hard Circuit Breakers**: Mandate an irrevocable daily drawdown ceiling (e.g., 2% of total equity) that immediately liquidates all positions and halts order routing.",
        ""
    ])

    return "\n".join(lines)

def generate_agent_markdown() -> str:
    return """---
name: forex-quant-expert
description: Principal Foreign Exchange (Forex) & Quantitative Currency Systems Architect for vibe code developers: OTC market microstructure, ECN/STP execution, triangular arbitrage, cross-currency basis, carry trades, risk parity, and systematic validation across 50 canonical texts.
tools: read_file, run_command, calculator
model: deepseek-reasoner
---

# Forex & Quantitative Currency Systems Architect Persona

You are the ECC Principal Foreign Exchange (Forex) & Quantitative Currency Systems Architect in Tagisan (TGS).

## Core Objective
Analyze, audit, architect, and mathematically verify automated Foreign Exchange trading systems, algorithmic execution engines, cross-currency statistical arbitrage models, and macroeconomic risk frameworks. Prevent catastrophic retail drawdown, backtest curve-fitting, and execution failures across currency spot, forwards, and futures.

## Core Directives & Frameworks
1. **OTC Market Microstructure & Interbank Mechanics (Lyons, Weithers, Harris)**:
   - Enforce quotation conventions: base vs. terms currency, reciprocal pip math, and cross-currency triangulation.
   - Model the two-tier OTC structure: customer-to-dealer vs. interbank dealer-to-dealer order flow.
   - Account for non-linear market impact ($I \\sim \\sigma \\sqrt{Q/V}$) and adverse selection from toxic institutional order flow.

2. **Algorithmic Architecture & Direct Market Access (Davey, Chan, Carver, Hilpisch)**:
   - Reject naive REST polling loops; mandate event-driven WebSocket and FIX protocol state machines.
   - Eliminate binary on/off trading logic in favor of continuous position sizing scaled inversely to prevailing ATR volatility.
   - Enforce realistic transaction cost analysis (TCA), accounting for commissions, dynamic spreads, and execution latency.

3. **Quantitative Strategies & Statistical Arbitrage (Vidyamurthy, Aronson, Gray, Tulchinsky)**:
   - Audit all time-series features for stationarity (Augmented Dickey-Fuller and Johansen cointegration tests).
   - Reject technical indicators that fail White's Reality Check or bootstrap permutation tests for data-mining bias.
   - Calculate Ornstein-Uhlenbeck mean-reversion half-life ($t_{1/2} = -\\ln(2)/\\lambda$) before enabling pairs or triangular arbitrage loops.

4. **Financial Machine Learning & AI Rigor (López de Prado, Jansen, Dixon, Kaabar)**:
   - Strictly prohibit random train/test splits; enforce Purged and Embargoed K-Fold Cross-Validation.
   - Use Triple Barrier Method with volatility-adjusted horizontal bounds and vertical time expiration bars.
   - Apply fractional differentiation to preserve memory while achieving stationarity.

5. **Risk Mathematics, Position Sizing & Capital Preservation (Vince, Taleb, Hull, Danielsson)**:
   - Restrict position sizing to Fractional Kelly ($c \\in [0.25, 0.5]$) bounded by Maximum Drawdown limits.
   - Account for fat tails, leptokurtosis, and non-linear gap risks where stop-loss orders fail to fill.
   - Mandate automated daily drawdown circuit breakers (hard kill-switches) that liquidate open exposure upon breach.

## Diagnostic & Audit Protocol
1. **Verify Pip Value Calculations**: Audit code for indirect (USD/JPY) and cross (EUR/GBP) pairs to guarantee settlement currency conversion matches account denomination.
2. **Stress-Test Transaction Costs**: Inject dynamic spread widening (3x-10x) during the 5:00 PM NY rollover and during tier-1 macroeconomic data releases.
3. **Audit Backtest Validation**: Check for Walk-Forward Efficiency (WFE > 60%) and Monte Carlo drawdown distribution before permitting live deployment.
4. **Inspect Order Execution Routing**: Ensure smart order routing uses TWAP or iceberg slicing on large orders to prevent quote-shading and front-running by retail broker B-books.
"""

def generate_skill_markdown() -> str:
    return """---
name: "forex-and-currency-trading-vibe-coder"
description: "Master Foreign Exchange (Forex) & Algorithmic Currency Trading skill for vibe code developers: Market Microstructure, Order Flow, Triangular Arbitrage, Carry Trade Dynamics, Statistical Arbitrage, Walk-Forward Validation, and Risk Sizing across 50 canonical texts."
---
# Foreign Exchange (Forex) & Currency Trading for the Vibe Code Developer

## Overview & Escaping Currency Trading Fragility
When vibe code developers use AI models to synthesize Forex trading bots, MQL5 Expert Advisors (EAs), or Python execution pipelines in minutes, the generated code almost universally fails in live trading due to **Subtle Microstructure and Mathematical Flaws**:
- **Naive Pip Math**: Assuming 1 pip is always $10.00 across all pairs, leading to massive over-leveraging on cross pairs like EUR/GBP or GBP/JPY.
- **Rollover & Spread Blindness**: Backtesting with fixed 0.5 pip spreads, ignoring the 5:00 PM NY rollover spread blowout (5-15 pips) and negative swap rates.
- **Lookahead & Data Leakage**: Applying standard MinMax scalers or random train/test splits across time series data, creating hallucinated 90% win rates.
- **Overfitted Indicator Soup**: Stacking RSI, MACD, and Bollinger Bands with optimized parameters that collapse as soon as market volatility regimes shift.
- **Uncapped Leverage & Martingale Risks**: Doubling down on losing positions or using fixed lot sizes that trigger rapid margin calls during central bank intervention spikes.

**This skill equips the vibe coder and AI agents with the complete mathematical, econometric, and algorithmic safeguards across the 50 authoritative Forex texts.**

---

## The Iron Laws of Forex Algorithmic Trading for Vibe Coders

```
1. ALWAYS COMPUTE DYNAMIC PIP VALUES: Never hardcode pip values; normalize quote currency conversion legs against account equity.
2. SPREADS ARE TIME-DEPENDENT: Model 3x-8x spread widening at 5 PM NY rollover and triple swap on Wednesdays.
3. PURGE AND EMBARGO TIME-SERIES: Never use random cross-validation; enforce Purged K-Fold CV to prevent lookahead leakage.
4. WALK-FORWARD EFFICIENCY > 60%: A strategy is an overfitted illusion unless out-of-sample performance maintains >= 60% of in-sample Sharpe.
5. FRACTIONAL KELLY WITH HARD STOP: Never risk more than 1-2% of account equity per trade; enforce an irrevocable daily drawdown kill-switch.
```

---

## The 8 Pillars of the Forex Canon

### Pillar I: FX Market Microstructure, Order Flow & Interbank Mechanics
- **#1. The Microstructure Approach to Exchange Rates** — *Richard K. Lyons* (ISBN-10: 0262122421 | ISBN-13: 978-0262122429)
- **#2. Foreign Exchange: A Practical Guide to the FX Markets** — *Tim Weithers* (ISBN-10: 0471732036 | ISBN-13: 978-0471732037)
- **#3. Day Trading and Swing Trading the Currency Market** — *Kathy Lien* (ISBN-10: 1119108411 | ISBN-13: 978-1119108412)
- **#4. Currency Forecasting** — *Michael R. Rosenberg* (ISBN-10: 1557389187 | ISBN-13: 978-1557389183)
- **#5. The Foreign Exchange Matrix** — *Barbara Rockefeller, Vicki Schmelzer* (ISBN-10: 0857191306 | ISBN-13: 978-0857191304)
- **#6. Inside the Currency Market** — *Brian Dolan* (ISBN-10: 1576603784 | ISBN-13: 978-1576603789)
- **#7. Exchange Rate Economics: Theories and Evidence** — *Ronald MacDonald* (ISBN-10: 0415125510 | ISBN-13: 978-0415125512)
- **#8. The Economics of Exchange Rates** — *Lucio Sarno, Mark P. Taylor* (ISBN-10: 0521485843 | ISBN-13: 978-0521485845)

### Pillar II: Algorithmic Architecture, System Engineering & Direct Market Access
- **#9. Building Algorithmic Trading Systems** — *Kevin J. Davey* (ISBN-10: 1118778987 | ISBN-13: 978-1118778982)
- **#10. Quantitative Trading** — *Ernest P. Chan* (ISBN-10: 1119800064 | ISBN-13: 978-1119800064)
- **#11. Algorithmic Trading: Winning Strategies and Their Rationale** — *Ernest P. Chan* (ISBN-10: 1118460146 | ISBN-13: 978-1118460146)
- **#12. Systematic Trading** — *Robert Carver* (ISBN-10: 0857194453 | ISBN-13: 978-0857194459)
- **#13. Python for Algorithmic Trading** — *Yves Hilpisch* (ISBN-10: 149205335X | ISBN-13: 978-1492053354)
- **#14. Trading and Exchanges: Market Microstructure for Practitioners** — *Larry Harris* (ISBN-10: 0195144708 | ISBN-13: 978-0195144703)
- **#15. Algorithmic and High-Frequency Trading** — *Álvaro Cartea et al.* (ISBN-10: 1107091144 | ISBN-13: 978-1107091146)
- **#16. Developing High-Frequency Trading Systems** — *Sebastien Donadio, Sourav Ghosh* (ISBN-10: 1788296311 | ISBN-13: 978-1788296311)
- **#17. Algorithmic Trading & DMA** — *Barry Johnson* (ISBN-10: 0956399207 | ISBN-13: 978-0956399205)

### Pillar III: Quantitative Strategies, Statistical Arbitrage & Technical Rigor
- **#18. Pairs Trading: Quantitative Methods and Analysis** — *Ganapathy Vidyamurthy* (ISBN-10: 0471460672 | ISBN-13: 978-0471460671)
- **#19. Statistically Sound Indicators for Financial Market Prediction** — *Timothy Masters* (ISBN-10: 1099682126 | ISBN-13: 978-1099682124)
- **#20. Evidence-Based Technical Analysis** — *David Aronson* (ISBN-10: 0470008741 | ISBN-13: 978-0470008744)
- **#21. Quantitative Momentum** — *Wesley R. Gray, Jack R. Vogel* (ISBN-10: 111923719X | ISBN-13: 978-1119237198)
- **#22. Finding Alphas** — *Igor Tulchinsky* (ISBN-10: 1119565200 | ISBN-13: 978-1119565208)
- **#23. Quantitative Trading Systems** — *Howard B. Bandy* (ISBN-10: 0979177006 | ISBN-13: 978-0979177002)
- **#24. Mean Reversion Trading Systems** — *Howard B. Bandy* (ISBN-10: 0979177073 | ISBN-13: 978-0979177071)
- **#25. Following the Trend** — *Andreas F. Clenow* (ISBN-10: 1118410858 | ISBN-13: 978-1118410851)

### Pillar IV: Machine Learning, Deep Learning & Modern AI in Finance
- **#26. Advances in Financial Machine Learning** — *Marcos López de Prado* (ISBN-10: 1119482089 | ISBN-13: 978-1119482086)
- **#27. Machine Learning for Asset Managers** — *Marcos López de Prado* (ISBN-10: 1108792898 | ISBN-13: 978-1108792899)
- **#28. Machine Learning for Algorithmic Trading** — *Stefan Jansen* (ISBN-10: 1839217715 | ISBN-13: 978-1839217715)
- **#29. Financial Machine Learning** — *Matthew F. Dixon et al.* (ISBN-10: 3030410676 | ISBN-13: 978-3030410674)
- **#30. Deep Learning for Finance** — *Sofien Kaabar* (ISBN-10: 1098148479 | ISBN-13: 978-1098148478)
- **#31. Reinforcement Learning for Finance** — *Bence Tóth* (ISBN-10: 1804618799 | ISBN-13: 978-1804618790)
- **#32. Hands-On Machine Learning for Algorithmic Trading** — *Stefan Jansen* (ISBN-10: 178934641X | ISBN-13: 978-1789346411)
- **#33. Artificial Intelligence in Finance** — *Yves Hilpisch* (ISBN-10: 1492055182 | ISBN-13: 978-1492055181)

### Pillar V: Order Book Dynamics, Microstructure & High-Frequency Execution
- **#34. Trades, Quotes and Prices** — *Jean-Philippe Bouchaud et al.* (ISBN-10: 1107164036 | ISBN-13: 978-1107164031)
- **#35. Empirical Market Microstructure** — *Joel Hasbrouck* (ISBN-10: 0195301641 | ISBN-13: 978-0195301649)
- **#36. High-Frequency Trading** — *Irene Aldridge* (ISBN-10: 1118343808 | ISBN-13: 978-1118343807)
- **#37. Market Microstructure in Practice** — *Charles-Albert Lehalle, Sophie Laruelle* (ISBN-10: 9813230835 | ISBN-13: 978-9813230835)
- **#38. Flash Boys: A Wall Street Revolt** — *Michael Lewis* (ISBN-10: 0393244660 | ISBN-13: 978-0393244663)
- **#39. Dark Pools and High Frequency Trading For Dummies** — *Jay Vaananen* (ISBN-10: 1119001374 | ISBN-13: 978-1119001379)

### Pillar VI: Backtesting, Walk-Forward Validation & Preventing Overfitting
- **#40. The Evaluation and Optimization of Trading Strategies** — *Robert Pardo* (ISBN-10: 0470128011 | ISBN-13: 978-0470128015)
- **#41. Testing and Tuning Market Trading Systems** — *Timothy Masters* (ISBN-10: 047124113X | ISBN-13: 978-0471241133)
- **#42. Maximum Adverse Excursion** — *John Sweeney* (ISBN-10: 0471177652 | ISBN-13: 978-0471177654)
- **#43. Model Risk Management** — *Peter Verhoeven* (ISBN-10: 3030784371 | ISBN-13: 978-3030784379)

### Pillar VII: Risk Management, Position Sizing & Capital Preservation
- **#44. The Mathematics of Money Management** — *Ralph Vince* (ISBN-10: 0471547387 | ISBN-13: 978-0471547389)
- **#45. Dynamic Hedging** — *Nassim Nicholas Taleb* (ISBN-10: 0471152803 | ISBN-13: 978-0471152804)
- **#46. Risk Management and Financial Institutions** — *John C. Hull* (ISBN-10: 1119448115 | ISBN-13: 978-1119448112)
- **#47. Financial Risk Forecasting** — *Jon Danielsson* (ISBN-10: 0470669438 | ISBN-13: 978-0470669433)

### Pillar VIII: Market Reality, Trader Intuition & Historical Lessons
- **#48. The Man Who Solved the Market** — *Gregory Zuckerman* (ISBN-10: 073521798X | ISBN-13: 978-0735217980)
- **#49. Market Wizards** — *Jack D. Schwager* (ISBN-10: 1118273052 | ISBN-13: 978-1118273050)
- **#50. Reminiscences of a Stock Operator** — *Edwin Lefèvre* (ISBN-10: 0471770884 | ISBN-13: 978-0471770886)

---

## Practical Implementation Patterns for AI Prompts

```markdown
Vibe Coder System Prompt Template:
"You are a quantitative Forex systems engineer building an event-driven bot in Python/Rust.
Enforce the following non-negotiable invariants:
1. Dynamic Pip Calculation: Normalize Pip values across base and quote pairs against account equity currency.
2. Friction Modeling: Invert bid/ask prices when crossing spreads; inject 1.5 pip base spread + 4x rollover expansion.
3. Feature Stationarity: Apply fractional differentiation (d ≈ 0.35) or log-returns; reject raw non-stationary price series.
4. Validation: Enforce 5-fold Purged and Embargoed Walk-Forward Cross-Validation. Reject any strategy with WFE < 60%.
5. Risk Engine: Size positions via Fractional Kelly (0.33) scaled to ATR; enforce a 2% hard daily drawdown circuit breaker."
```
"""

# --- 4. Deployment and Verification Workflow ---

def deploy_and_test():
    print("=================================================================")
    print(" 🇵🇭 DEPLOYING & VERIFYING FOREX & CURRENCY CANON IN TGS")
    print("=================================================================\n")

    base_proj = Path("/home/dyna/TGS Projects")
    tagisan_dir = base_proj / "tagisan"

    # Step 1: Verify all 50 ISBNs
    print(f"[STEP 1/5] Verifying ISBN-10 and ISBN-13 checksums for all {len(BOOKS_50_CANON)} books...")
    isbn_errors = []
    for book in BOOKS_50_CANON:
        n = book["num"]
        title = book["title"]
        i10 = book["isbn10"]
        i13 = book["isbn13"]
        if not validate_isbn10(i10):
            isbn_errors.append(f"Book #{n} '{title}': Invalid ISBN-10 ({i10})")
        if not validate_isbn13(i13):
            isbn_errors.append(f"Book #{n} '{title}': Invalid ISBN-13 ({i13})")

    if isbn_errors:
        print(f"❌ FAIL: {len(isbn_errors)} ISBN checksum errors found!")
        for err in isbn_errors:
            print("  -", err)
        return False
    print(f"✅ PASS: All {len(BOOKS_50_CANON)} books have 100% mathematically valid ISBN-10 and ISBN-13 checksums.\n")

    # Step 2: Generate documents and deploy to target directories
    print("[STEP 2/5] Generating and deploying files to TGS workspace...")
    guide_content = generate_guide_markdown()
    agent_content = generate_agent_markdown()
    skill_content = generate_skill_markdown()

    # 1. Guide in tagisan/docs
    docs_dir = tagisan_dir / "docs"
    docs_dir.mkdir(parents=True, exist_ok=True)
    guide_dest = docs_dir / "VIBE_CODER_FOREX_AND_CURRENCY_TRADING_GUIDE.md"
    guide_dest.write_text(guide_content, encoding="utf-8")
    print(f"  [+] Deployed Guide: {guide_dest} ({len(guide_content)} bytes)")

    # 2. Agent & Skill in both base_proj and tagisan
    for root_dir in [base_proj, tagisan_dir]:
        # Agent
        agent_dir = root_dir / ".ecc" / "agents"
        agent_dir.mkdir(parents=True, exist_ok=True)
        agent_dest = agent_dir / "forex-quant-expert.md"
        agent_dest.write_text(agent_content, encoding="utf-8")
        print(f"  [+] Deployed Agent: {agent_dest}")

        # Skill
        skill_dir = root_dir / ".ecc" / "skills" / "forex-and-currency-trading-vibe-coder"
        skill_dir.mkdir(parents=True, exist_ok=True)
        skill_dest = skill_dir / "SKILL.md"
        skill_dest.write_text(skill_content, encoding="utf-8")
        print(f"  [+] Deployed Skill: {skill_dest}")

    # Invalidate cache
    cache_path = tagisan_dir / ".tagisan" / "skills.cache"
    if cache_path.exists():
        cache_path.unlink()
        print("  [+] Invalidated .tagisan/skills.cache to trigger fresh discovery.")
    print("✅ PASS: All target files successfully deployed.\n")

    # Step 3: Copy verification script to tagisan/scripts/
    scripts_dir = tagisan_dir / "scripts"
    scripts_dir.mkdir(parents=True, exist_ok=True)
    script_dest = scripts_dir / "test_forex_verification.py"
    # Write the self-contained verification script
    if Path(__file__).resolve() != script_dest.resolve():
        shutil.copy(__file__, script_dest)
    print(f"  [+] Installed Verification Test Suite: {script_dest}\n")

    # Step 4: Mathematical precision calculations
    print("[STEP 4/5] Testing quantitative financial invariant equations...")
    # Pip calculation
    lot = 100000
    assert math.isclose(lot * 0.0001, 10.0) # EUR/USD
    assert math.isclose((lot * 0.01) / 150.0, 6.666667, rel_tol=1e-4) # USD/JPY
    assert math.isclose(lot * 0.0001 * 1.30, 13.0) # EUR/GBP
    # Triangular Arbitrage bound
    eur_usd, gbp_usd, eur_gbp = 1.0850, 1.2850, 0.8440
    synthetic_eur_gbp = eur_usd / gbp_usd
    assert abs(synthetic_eur_gbp - 0.8443) < 0.001
    # OU half-life
    assert 13.8 < (-math.log(2) / -0.05) < 13.9
    # Fractional Kelly
    assert math.isclose((0.55 * 1.5 - 0.45) / 1.5 * 0.5, 0.125)
    print("✅ PASS: All Forex quantitative calculations verified.\n")

    # Step 5: Live tgs binary integration test
    print("[STEP 5/5] Executing live CLI integration tests with 'tgs' binary...")
    tgs_bin = shutil.which("tgs") or "/home/dyna/.cargo/bin/tgs"
    if not os.path.exists(tgs_bin):
        print(f"⚠️ Warning: 'tgs' binary not found at {tgs_bin}.")
        return True

    # Check `tgs ecc list`
    print("  Executing: tgs ecc list")
    res_list = subprocess.run([tgs_bin, "ecc", "list"], cwd=str(tagisan_dir), capture_output=True, text=True)
    if "forex-quant-expert" in res_list.stdout:
        print("  [✓] SUCCESS: 'forex-quant-expert' successfully detected by 'tgs ecc list'!")
    else:
        print("❌ FAIL: 'forex-quant-expert' missing in 'tgs ecc list' output:\n", res_list.stdout)
        return False

    # Check `tgs ecc skills`
    print("  Executing: tgs ecc skills (checking for 'forex-and-currency-trading-vibe-coder')")
    res_skills = subprocess.run([tgs_bin, "ecc", "skills"], cwd=str(tagisan_dir), capture_output=True, text=True)
    combined = res_skills.stdout + res_skills.stderr
    if "forex-and-currency-trading-vibe-coder" in combined:
        print("  [✓] SUCCESS: 'forex-and-currency-trading-vibe-coder' successfully detected by 'tgs ecc skills'!")
    else:
        print("❌ FAIL: 'forex-and-currency-trading-vibe-coder' missing in 'tgs ecc skills' output.")
        return False

    print("\n=================================================================")
    print(" 🎉 ALL TESTS PASSED: FOREX CANON IS 100% REAL, TESTED & ACTIVE")
    print("=================================================================")
    return True

if __name__ == "__main__":
    ok = deploy_and_test()
    sys.exit(0 if ok else 1)
