# <h1 align="center"> Hyperlane Blueprint Template 🌐 </h1>

## 📚 Overview
This repo contains a templated AVS Blueprint for a Hyperlane node. It contains tasks for an operator to manage their own Hyperlane node and aims to expose both operator-centric and user-centric tasks.

## 🚀 Features

This Hyperlane Blueprint Template provides the following key feature:

### 1. Operate a Warp Route
- **Function**: `operate_a_warp_route`
- **Description**: Initializes and operates a Hyperlane warp route, including deploying Hyperlane contracts, initializing the warp route, updating core configurations, and then validating/operating over the route.
- **Job ID**: 0
- **Parameters**:
  - `config`: Configuration for the warp route (Vec<u8>)
  - `advanced`: Boolean flag for advanced setup
  - `use_existing_core_config`: Existing core configuration (Vec<u8>)

This job is designed to be instanced as part of the Tangle network's Cloud, allowing for decentralized management and operation of Hyperlane nodes and warp routes.

## 🔧 Usage

To interact with this job, you'll need to deploy this blueprint to Tangle. Upon deployment, the Blueprint will be able to be instanced and executed by any Tangle operator registered on the Hyperlane Blueprint. The job can be triggered by sending the appropriate transaction to the Tangle network, specifying the job ID and required parameters.

For example, to operate a warp route, you would prepare a transaction with job ID 0 and include the configuration, advanced flag, and existing core configuration (if any) as parameters.

Please refer to the Tangle network documentation for detailed instructions on how to submit jobs and interact with AVS Blueprints.

## 📚 Prerequisites

Before you can run this project, you will need to have the following software installed on your machine:

- [Rust](https://www.rust-lang.org/tools/install)
- [Tangle](https://github.com/tangle-network/tangle?tab=readme-ov-file#-getting-started-)


## 🛠️ Build

To build the project, run:

```
cargo build --release
```