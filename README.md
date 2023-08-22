# CANParser: CAN Parsing Library & CLI Utility

CANParser is a robust, Rust-based CAN parsing library, designed with speed and accuracy in mind. Developed in Rust for its efficiency, direct bit-level manipulation, and vast compatibility with various architectures and languages. It aims to parse CAN (Controller Area Network) log files, decode them based on provided specifications, and produce meaningful output. Additionally, it provides a command-line interface utility for effortless integration into terminal-based workflows.

## Table of Contents

1. [Features](#features)
2. [Repository Structure](#repository-structure)
3. [Library Features](#library-features)
4. [Data Variables](#data-variables)
5. [Main Functions](#main-functions)
6. [Getting Started](#getting-started)
7. [Benchmarking Statistics](#benchmarking-statistics)
8. [Memory Estimates and Limitations](#memory-estimates-and-limitations)
9. [Examples](#examples)
10. [Future Work](#future-work)

## Features
- **Blazing Fast**: Optimized for speed, providing parse times as low as ~700ns per line on modern systems.
- **Versatile Input**: Support for all text-based CAN logs. Specification files can be in JSON, XLSX, or DBC formats.
- **Diverse Output Options**: Output can be formatted in JSON, CSV, or as an SQLite database.
- **Broad Compatibility**: With included wrappers, it supports integration with Python and web clients through WebAssembly.

## Repository Structure
- `can_parser/`: Contains the Rust-based CAN parsing utility.
- `can_parser_python/`: Features a pyo3 wrapper, allowing the utility to be compiled into a Python library.
- `can_parser_wasm/`: Contains a wasm_bindgen wrapper, permitting compilation into web assembly for client-side web integrations.
- `can_parser_cli`: Provides a Rust-based CLI utility, which also serves as a Rust example for utilizing the CAN parsing library.
- `examples/`: Includes Python and Next.js usage samples.

## Features in Detail
- `parallel`: Enables multi-threaded parsing.
- `debug`: Activates debug output, including benchmarking.
- `xlsx`: Support for Microsoft XLSX document specifications.
- `sqlite`: Enables SQLite database output.
- `python`: PyO3 support for Python versions 3.7 and above.
- `wasm`: Enables wasm-bindgen support. (Note: multi-threaded support for WASM requires the nightly unstable std, resulting in potential instability).

## Primary Data Variables
- `messages`: Array of `CANMessages` where parsed data values reside.
- `filtered_spec`: A filtered specification for items in the CAN logs.
- `flags`: Boolean flags indicating detected protocols in the CAN log.

> **NOTE**: Multi-line parsing functions don't return data directly to avoid unnecessary data copy when using wrapper libraries. They return success status; to access data, refer to class variables.

## Key Functions
(For brevity, not all functions are detailed here. Refer to the library documentation for full details.)

- **Constructor**: Initializes a `CANParser` instance. Excerpt:
  ```rust
  pub fn new(
      error_handling: String,
      line_regex: Option<String>,
      specs_annexes: Option<HashMap<String, String>>,
  ) -> Result<Self, CANParserError>;
  ```
  
- **parse_file**: Parses a file, returning the operation's success status.
- **parse_lines**: Parses an array of lines.
- **parse_line**: Parses a single CAN message and directly returns the parsed message.
- **to_json**: Outputs `filtered_spec`, `flags`, and `messages` as a JSON string or saves them to a specified file.
- **to_csv**: Converts data to multiple CSVs.
- **to_sqlite**: Stores data in an SQLite database, the most memory-efficient option.

## Getting Started

Before diving into the core functionalities of CANParser, it's essential to set up your development environment. Follow these steps to get started:

### Prerequisites:

1. **Git**: To clone the repository, ensure that you have Git installed on your machine. If not, you can download and install it from the official [Git website](https://git-scm.com/).

### Cloning the Repository:

1. Open your terminal or command prompt.
2. Navigate to the directory where you want to clone the CANParser repository.
3. Execute the following command:
    ```bash
    git clone https://github.com/systemscyber/CANParser.git
    ```
4. Change to the cloned directory:
    ```bash
    cd CANParser
    ```

### Installing Rust:

Rust provides excellent documentation for setting up the language on various operating systems. Here's a quick guide:

#### For Linux:

1. Open your terminal.
2. Download and install Rust using the rustup script:
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
3. The above command will download a script and start the installation of the rustup toolchain installer. Follow the on-screen instructions.
4. Once completed, restart your terminal or run the following command to update your path:
    ```bash
    source $HOME/.cargo/env
    ```
5. Verify the installation by executing:
    ```bash
    rustc --version
    ```

#### For Windows:

1. Visit the official [Rust downloads page](https://www.rust-lang.org/tools/install).
2. Download the `rustup-init.exe` for Windows.
3. Run the downloaded executable and follow the on-screen instructions.
4. After installation, restart your computer.
5. Open a new Command Prompt and verify the installation:
    ```bash
    rustc --version
    ```

## Building CANParser

Building CANParser involves compiling and setting up different components, namely the CLI, Python library, and the WASM library. Here’s a detailed guide for each component:

### Building the CLI:

The CLI provides a command-line interface to interact with the CANParser library. 

1. Navigate to the `can_parser_cli` directory:
    ```bash
    cd can_parser_cli
    ```

2. **Using the Makefile** (Recommended for Unix-like systems):

    - Compile the binary for your system in release mode:
      ```bash
      make
      ```

    - To compile for multiple targets (Windows, Linux, Raspberry Pi, and BeagleBone):
      ```bash
      make all
      ```

3. **Manual Compilation**:

    If you are on Windows or don’t prefer using the Makefile, you can compile manually using Cargo.

    - Compile the binary in release mode:
      ```bash
      cargo build --release
      ```

### Building the Python Library:

1. First, ensure you have Python installed. For this library, we're targeting Python 3.7 and higher.

2. Create and activate a virtual environment:

    #### For Linux:
    ```bash
    python3 -m venv canparser_env
    source canparser_env/bin/activate
    ```

    #### For Windows:
    ```bash
    python -m venv canparser_env
    .\canparser_env\Scripts\activate
    ```

3. Install `maturin`, a tool to build and publish Rust projects with Python bindings:
    ```bash
    pip install maturin
    ```

4. Navigate to the `can_parser_python` directory and build the library:
    ```bash
    cd can_parser_python
    maturin build --release
    ```

5. To automatically compile and install the Python wheel in the current environment, use:
    ```bash
    maturin develop --release
    ```

### Building the WASM Library:

1. Install `wasm-pack`, a toolchain for building WebAssembly with Rust:
    ```bash
    cargo install wasm-pack
    ```

2. Navigate to the `can_parser_wasm` directory:
    ```bash
    cd can_parser_wasm
    ```

3. Build the package targeting web platforms:
    ```bash
    wasm-pack build --target web
    ```

After these steps, you've successfully built the various components of CANParser. Now, you can proceed to use them in your applications or further development. Always refer back to the provided examples for practical implementation insights.

## Benchmarking Statistics
*To view the benchmark statistics enabled the "debug" feature and then recompile.*

Detailed benchmarking stats help users get a sense of the library's performance. For instance, parsing a 230MB candump log on a modern four-core system with multi-threaded support took 3.26 seconds, averaging 704 ns per line.
```
Debug Information:
------------------
File Parsing Duration: 3.26 seconds
Total Lines Parsed: 4630148
Average Time per Line: 704 ns
Average Message Size: 144.00 bytes
------------------
```
At the time of writing this here are some other statistics (as of 8/2023) on a relatively modern machine across a few different can logs:
- Single threaded average time per line: ~2us (micro seconds)
- Single threaded Firefox average time per line: 2us
- Single threaded Chrome average time per line: 16us
- Single threaded Edge average time per line: 32us

## Memory Estimates and Limitations
As shown in the debug information above a decoded CANMessage in memory takes about 144 bytes. The typical can message in an ascii candump log file takes around 50 bytes. As such for text based log files you'll need roughly around 6x the size of the file free space in memory:
```
Ex: size of the file = 1
the_file(1) + the_spec_file_in_memory(1) + the_decoded_messages(3) + filtered_spec_in_memory(1) = 6 (Again its a rough estimate)
```
You'll also need more space to write the results to disk. How much will largely depend on your method of output.

### Browser limitations:
As mentioned before the multithreaded version of the wasm wrapper requires the nightly unstable std library. This appears to work well for can logs under 10MB but it starts getting memory errors and exceptions for larger logs. As such its recommended to use the single threaded version. The single threaded version is still limited by the browser at least compared to native performance. It has successfully parsed files hundreds of MBs in size running in the browser but depending on the system, browser, and browser settings it may perform differently.

## Examples
In the examples folder there is a python example showing python using the parsing library. Be sure to have compiled and installed the can_parser_python wheel before running the example.
In addition there is a simple nextjs example project showing the wasm wrapper being used on a website. In this example the parser is ran in a separate web worker so that its not running it in the same worker that the graphics are being run on. This is not a requirement (it is for the multithreaded version) but it is a recommendation.

## Future Work
Features like OpenDBC support, UDS Parsing, Transport Session parsing, binary log parsing, and efficient chunk based parsing that can handle files that are too large to fit into memory are under consideration.

---

Contributions, suggestions, and insights are always welcome.