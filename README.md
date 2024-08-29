# ScreenStreamX

## Overview

ScreenStreamX is a multi-platform screen-casting application developed in Rust. The application is designed to continuously capture the content of your screen (or a portion of it) and stream it to a set of peers. It is compatible with Windows, macOS, and Linux, and features a user-friendly interface for seamless navigation.

## Features

### Core Features

1. **Platform Support**: Compatible with Windows, macOS, and Linux.
2. **User Interface (UI)**: Intuitive and user-friendly interface.
3. **Operating Mode**:
    - **Caster Mode**: Streams the screen content.
    - **Receiver Mode**: Connects to a caster and displays the streamed content. Users can specify the address of the caster to connect to.
4. **Selection Options**: In caster mode, users can restrict the captured content to a custom area of the screen.
5. **Hotkey Support**: Customizable keyboard shortcuts for:
    - Pausing/resuming the transmission.
    - Blanking the screen.
    - Terminating the current session.

### Bonus Features

6. **Annotation Tools**: In caster mode, users can activate/deactivate a transparent layer on top of the captured area to superimpose annotations such as shapes, arrows, and text.
7. **Save Options**: In receiver mode, users can record the received content to a video file.
8. **Multi-Monitor Support**: The application can recognize and handle multiple monitors independently, allowing users to cast content from any connected display.

## Getting Started

### Prerequisites

- Rust programming language installed on your machine.
- Compatible operating system (Windows, macOS, or Linux).

### Installation

1. Clone the repository:
    ```sh
    git clone https://github.com/your-username/screenstreamx.git
    cd screenstreamx
    ```

2. Build the project:
    ```sh
    cargo build --release
    ```

### Running the Application

1. Start the application:
    ```sh
    cargo run --release
    ```

2. Choose the operating mode (caster or receiver) upon startup.
3. Follow the on-screen instructions to either start casting your screen or connect to a caster.

### Customizing Hotkeys

Hotkeys can be customized through the application settings menu. Default hotkeys include:
- **Pause/Resume**: `Ctrl + Alt + P`
- **Blank Screen**: `Ctrl + Alt + B`
- **Terminate Session**: `Ctrl + Alt + T`

## Contributing

We welcome contributions to enhance the functionality and usability of this application. To contribute, please follow these steps:

1. Fork the repository.
2. Create a new branch:
    ```sh
    git checkout -b feature/your-feature-name
    ```
3. Make your changes and commit them:
    ```sh
    git commit -m "Add your commit message"
    ```
4. Push to the branch:
    ```sh
    git push origin feature/your-feature-name
    ```
5. Open a pull request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Acknowledgements

We would like to thank the contributors and the Rust community for their support and valuable contributions.

---

For more details and documentation, please visit the [Wiki](https://github.com/your-username/screenstreamx/wiki) page. If you encounter any issues or have any questions, feel free to open an issue on GitHub.

## TO DO
Implementare annotazioni  -> lato caster 