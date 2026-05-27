# Atlas ERP Architecture

## Overview

Atlas is a declarative ERP system built in Rust. It follows an evolutionary architecture, starting as a modular monolith and transitioning towards a microservices-based system.

## Core Principles

### 1. Everything as Data
Data models, workflows, validations, and security policies are defined as declarative data structures, not hardcoded logic.

### 2. Modular Monolith
The system is currently structured as a modular monolith within a single Cargo workspace. 
- `atlas-core`: Contains the generic declarative engines (Schema, Workflow, etc.) and domain-specific logic (to be decoupled).
- `atlas-gateway`: The primary entry point and host for the monolith.
- `atlas-shared`: Shared types and utilities used across all modules.
- Domain Crates (`atlas-hcm`, `atlas-financials`, etc.): Thin wrappers for domain-specific logic.

### 3. Hot-Reloading
The system is designed to pick up configuration changes from the database without requiring service restarts.

## Key Components

### Schema Engine
Manages dynamic entity definitions and field types.

### Workflow Engine
Executes declarative state machines for business processes.

### Event Bus
NATS-based event system for inter-module communication.

## Future Roadmap

The long-term goal is to decouple domain-specific logic from `atlas-core` and deploy them as independent microservices.
