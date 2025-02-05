

# **ScreenStreamX**

## **Overview**

**ScreenStreamX** is a **multi-platform screen-sharing application** built in **Rust**. It allows users to **capture and stream their screen** (or a selected portion of it) to multiple receivers in real-time. The application supports **Windows, macOS, and Linux** and features an intuitive UI with customizable hotkeys.

## **Features**

### **🔹 Core Features**
- **Multi-platform support**: Works on **Windows, macOS, and Linux**.
- **User-friendly interface**: Easy-to-navigate UI with real-time control.
- **Operating Modes**:
  - **Caster Mode** 🎥: Streams the screen content to receivers.
  - **Receiver Mode** 📡: Connects to a caster and displays the streamed content.
- **Custom Capture Area**: Allows users to select a specific area of the screen to stream.
- **Hotkey Support**:
  - Start/stop streaming.
  - Pause/resume transmission.
  - Lock/unlock the screen.
  - End the session.

### **🔹 Advanced Features**
- **Live Annotations** ✍️: In caster mode, users can draw and overlay annotations such as:
  - Lines, circles, rectangles, arrows.
  - Pencil and highlighter tools.
  - Text, eraser, and crop tools.
- **Screen Recording** 🎥: In receiver mode, users can record the received stream to a video file.
- **Multi-Monitor Support** 🖥️: Select and stream from any connected monitor.

---

## **📌 Getting Started**

### **🔹 Prerequisites**
- **Rust** installed on your system.
- A compatible operating system (**Windows/macOS/Linux**).

### **🔹 Installation**
1. Clone the repository:
   ```sh
   git clone https://github.com/your-username/screenstreamx.git
   cd screenstreamx
   ```
2. Build the project:
   ```sh
   cargo build --release
   ```

---

## **🚀 Running the Application**

1. Start the application:
   ```sh
   cargo run --release
   ```
2. Choose the operating mode: **Caster** (streaming) or **Receiver** (viewing).
3. Follow the on-screen instructions to either start a stream or connect to a caster.

---

## **🎯 Customizing Hotkeys**
Hotkeys can be modified through the application settings.  
**Default hotkeys include**:
- **Start/Pause Streaming**: `Ctrl + Shift + B`
- **Start/Stop Recording**: `Ctrl + Shift + R`
- **Lock/Unlock Screen**: `Ctrl + Shift + L`
- **Toggle Annotations**: `Ctrl + Shift + A`
- **Quick Capture Selection**: `Ctrl + Shift + S`
- **End Session**: `Ctrl + Shift + Q`
- **Switch Monitor**: `Ctrl + Shift + M`

---

## **🤝 Contributing**
We welcome contributions to enhance ScreenStreamX! Follow these steps:

1. **Fork** the repository.
2. Create a new branch:
   ```sh
   git checkout -b feature/your-feature-name
   ```
3. Make your changes and commit:
   ```sh
   git commit -m "Description of your changes"
   ```
4. Push to your branch:
   ```sh
   git push origin feature/your-feature-name
   ```
5. Open a **Pull Request**.

---

## **📜 License**
This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.

---

## **📢 Acknowledgements**
A big thanks to all contributors and the **Rust** community for their support.

📖 **For more details, check out the [Wiki](https://github.com/your-username/screenstreamx/wiki).**  
❓ **If you encounter issues, feel free to open a GitHub issue.** 🚀

---