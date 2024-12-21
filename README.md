
# PMD2-HWiNFO

**Real-time power monitoring tool bridging ElmorLabs PMD2 measurements to HWiNFO's sensor interface.**  
Built with ❤️ in Rust.

---
[![Releases](https://img.shields.io/github/v/release/lsx285/PMD2-HWiNFO?label=Latest%20Release&style=flat-square)](https://github.com/lsx285/PMD2-HWiNFO/releases/latest)  [![VirusTotal](https://img.shields.io/badge/VirusTotal-blue?style=flat-square)](https://www.virustotal.com/gui/file/1f7a63c60445d787a6b238af0a392e4f48e1e6366fff2549db1594a95779522b)
---

## Features

- **Real-time monitoring** of ElmorLabs PMD2
- **Seamless integration** with HWiNFO
- **Low resource usage**, ensuring minimal impact on system performance
- Supports monitoring of multiple power rails:
  - **ATX Rails:** 12V, 5V, 5VSB, and 3.3V
  - **12VHPWR**
  - **EPS Rails:** 1 & 2
  - **PCIe Rails:** 1-3

---

## Prerequisites

Ensure you have the following before starting:

- **Windows OS**
- [HWiNFO](https://www.hwinfo.com/)
- [ElmorLabs PMD2](https://elmorlabs.com/product/elmorlabs-pmd2/)

---

## Installation

1. **Download** the latest release from the [Releases Page](https://github.com/lsx285/PMD2-HWiNFO/releases/latest).
2. **Run** the downloaded executable.
3. The PMD2 sensors will now appear in HWiNFO under **"ElmorLabs PMD2"**.

---

## Usage

The application runs seamlessly in the background and automatically:
- Detects the **PMD2 device**.
- Reads **power**, **voltage**, and **current** measurements.
- Updates **HWiNFO sensor values** every 100ms.

**No configuration needed** – just run the tool and monitor in HWiNFO!

---

## Monitored Values

| **Category** | **Measurements**                          |
|--------------|-------------------------------------------|
| **Summary**  | Total Power, EPS Power, PCIe Power, MB Power |
| **Per Rail** | Voltage (V), Current (A), Power (W)       |

---

## Building from Source

Follow these steps to build the application from source:

```bash
# Clone the repository
git clone https://github.com/lsx285/PMD2-HWiNFO.git
cd PMD2-HWiNFO

# Build in release mode
cargo build --release

# Run without console window
cargo rustc --release --bin PMD2-HWiNFO -- -Clink-args="/SUBSYSTEM:WINDOWS /ENTRY:mainCRTStartup"
```

---

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

## Acknowledgments

Special thanks to:

- [ElmorLabs](https://elmorlabs.com/) for PMD2.
- [Martin Malík](https://www.hwinfo.com/) for HWiNFO and custom sensor API.
