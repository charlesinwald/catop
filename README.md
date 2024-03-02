---

# CatOp

CatOp is a terminal-based application designed to monitor system metrics like CPU and RAM usage, alongside displaying running processes in a tabular format. Built with Rust and leveraging the `tui` and `systemstat` crates, CatOp combines functionality with fun by animating an ASCII art cat whose behavior changes based on your system's CPU load.

![catop screenshot](https://imgur.com/a/i5S55Yk)

## Features

- **CPU Usage Monitoring**: Displays the current CPU load as a percentage.
- **RAM Usage Monitoring**: Shows the used RAM as a percentage of total available memory.
- **Process List**: Lists currently running processes with their PID, name, CPU usage, and memory footprint.
- **ASCII Cat Animation**: Features an ASCII cat that animates based on the CPU load, adding a playful element to system monitoring.

Quickly install `catop` by running the following command in your terminal:

```sh
curl -sSL -o install_catop.sh https://raw.githubusercontent.com/charlesinwald/catop/main/install.sh && chmod +x install_catop.sh && ./install_catop.sh
```

After installation, you might need to restart your terminal or source your shell configuration file:

- For Bash: `source ~/.bashrc`
- For Zsh: `source ~/.zshrc`

## Build from Source

Before you can run CatOp, you need to have Rust and Cargo installed on your machine. If you haven't installed Rust and Cargo yet, follow the instructions on [https://rustup.rs/](https://rustup.rs/) to set them up.

To install CatOp, clone the repository and build the project:

```bash
git clone https://github.com/charlesinwald/catop.git
cd catop
cargo build --release
```

After building, you can run the application from the project directory using Cargo:

```bash
cargo run
```

## Usage

Once CatOp is running, it will display your system's metrics in real-time. The application's UI is divided into sections for CPU usage, RAM usage, process list, and the ASCII cat animation.

- To **exit** the application, press `q` or `Esc`.

## Customization

CatOp allows for some customization, such as adjusting the ASCII cat animation or modifying the displayed metrics. To customize the ASCII cat, edit the `cat_frames` variable within the `animate_cat` function in `main.rs`.

## Contributing

Contributions to CatOp are welcome! Feel free to fork the repository, make your changes, and submit a pull request.

## License

CatOp is released under the MIT License. See the LICENSE file for more details.

## Acknowledgments

- The `tui` crate for providing an excellent TUI library.
- The `systemstat` crate for enabling system metrics collection.

---

Feel free to adjust the content to match your project's actual structure, usage, and contribution guidelines. This README provides a basic structure to help users understand and use your application.