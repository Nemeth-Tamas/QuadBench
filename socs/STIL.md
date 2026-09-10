# QuadBench — Betaflight SITL Setup

QuadBench runs natively on Windows.

Betaflight SITL runs as a native Linux x86_64 application, so the recommended Windows development setup is WSL2 with Ubuntu.

QuadBench communicates with SITL using:

| Direction | Protocol | Port |
| --- | --- | ---: |
| Betaflight → QuadBench | UDP motor output | 9002 |
| QuadBench → Betaflight | UDP FDM / sensors | 9003 |
| QuadBench → Betaflight | UDP RC channels | 9004 |
| QuadBench proxy → Betaflight | TCP MSP / UART1 | 5761 |
| Browser → QuadBench | WebSocket | 6761 |

---

## 1. Install WSL

From an elevated Windows PowerShell:

```powershell
wsl --install -d Ubuntu