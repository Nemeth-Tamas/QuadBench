# QuadBench — TODO

QuadBench is a Rust-based virtual quadcopter hardware bench for practicing and testing real Betaflight behavior without requiring a physical quad.

The long-term goal is to emulate enough of a real quadcopter hardware stack that Betaflight can interact with simulated receiver, GPS, ESC, battery, sensors, and other peripherals while QuadBench provides a live dashboard and fault-injection controls.

Real Betaflight should remain the flight-controller software wherever possible. QuadBench should simulate the hardware around it rather than reimplementing Betaflight behavior.

---

## Phase 0 — Project Foundation

* [x] Create initial Rust workspace
* [x] Define workspace/crate structure
* [ ] Add `README.md`
* [x] Add `.gitignore`
* [ ] Add Rust formatting configuration if required
* [x] Establish logging with `tracing`
* [ ] Establish error handling conventions
* [ ] Establish application configuration format
* [x] Add debug/release build profiles
* [x] Confirm clean build on Windows
* [ ] Confirm clean build on Linux
* [ ] Add basic CI build checks

Initial proposed structure:

```text
QuadBench/
├── Cargo.toml
├── README.md
├── TODO.md
├── crates/
│   ├── quadbench-app/
│   ├── quadbench-core/
│   ├── quadbench-betaflight/
│   ├── quadbench-input/
│   ├── quadbench-hardware/
│   └── quadbench-physics/
└── assets/
```

---

## Phase 1 — Application Skeleton

* [x] Create `quadbench-app`
* [x] Add `eframe` / `egui`
* [x] Create main application window
* [x] Add top-level tabs/pages

  * [x] Dashboard
  * [x] Receiver
  * [x] Motors / ESC
  * [x] Battery
  * [x] GPS
  * [x] Sensors
  * [x] Fault Injection
  * [x] Betaflight
  * [x] Logs
  * [x] Settings
* [x] Add application status bar
* [x] Add connection-state indicators
* [x] Add start/stop simulation controls
* [ ] Add configurable simulation update rate
* [ ] Add persistent settings

---

## Phase 2 — Shared Simulation State

* [ ] Create central `QuadState`
* [ ] Define receiver state
* [ ] Define motor state
* [ ] Define ESC state
* [ ] Define battery state
* [ ] Define GPS state
* [ ] Define gyro state
* [ ] Define accelerometer state
* [ ] Define attitude state
* [ ] Define flight-controller connection state
* [ ] Define simulation timing state
* [ ] Define arming state
* [ ] Define flight mode state
* [ ] Define failsafe state
* [ ] Define fault state
* [ ] Make simulator state thread-safe
* [ ] Separate commanded state from measured/simulated state

Suggested basic motor representation:

```text
Motor 1
Motor 2
Motor 3
Motor 4

command
normalized output
estimated RPM
current draw
temperature
fault status
```

---

## Phase 3 — Betaflight SITL Connection

* [ ] Document supported Betaflight SITL version
* [ ] Build or obtain Betaflight SITL locally
* [ ] Start Betaflight SITL manually
* [x] Connect QuadBench to Betaflight SITL
* [x] Implement SITL transport abstraction
* [x] Receive Betaflight motor outputs
* [x] Send simulated vehicle/sensor state
* [x] Send basic RC channel input
* [x] Detect SITL disconnect
* [x] Support reconnect without restarting QuadBench
* [x] Show SITL connection state in GUI
* [ ] Log raw SITL packet activity in debug mode
* [ ] Validate motor ordering against Betaflight
* [x] Validate channel ordering against Betaflight

Milestone:

```text
Betaflight Configurator
        ↕
Real Betaflight SITL
        ↕
QuadBench
```

Betaflight must be configurable through the normal Betaflight tooling rather than through a fake configuration implementation.

---

## Phase 4 — RadioMaster Pocket Input

* [x] Detect connected game controllers / HID devices
* [x] Detect RadioMaster Pocket when connected through USB
* [x] Display detected device information
* [x] Read Pocket stick channels
* [x] Read switches
* [x] Read buttons
* [x] Add channel monitor
* [ ] Add channel calibration
* [ ] Add deadband configuration
* [ ] Add channel inversion
* [ ] Add channel mapping
* [ ] Save controller mappings
* [ ] Handle controller disconnect
* [ ] Handle controller reconnect

Initial channel display:

```text
Aileron   1500
Elevator  1500
Throttle   988
Rudder    1500
AUX1      1000
AUX2      2000
...
```

---

## Phase 5 — Basic Receiver Simulation

* [x] Convert Pocket HID state into virtual receiver state
* [x] Implement simple SITL RC input first
* [x] Support at least 16 receiver channels internally
* [ ] Add configurable channel rate
* [x] Add receiver connected/disconnected state
* [ ] Add simulated RSSI
* [ ] Add simulated LQ
* [ ] Add packet-loss percentage
* [ ] Add packet jitter
* [ ] Add receiver latency
* [ ] Add forced RX loss
* [ ] Show failsafe state

Milestone:

Pocket USB input should be visible correctly in Betaflight's Receiver tab.

---

## Phase 6 — Motor / ESC Dashboard

* [ ] Receive four Betaflight motor commands
* [ ] Display Motor 1–4 output live
* [ ] Add horizontal motor output bars
* [ ] Add estimated RPM
* [ ] Add motor direction
* [ ] Add per-motor current draw
* [ ] Add per-ESC temperature
* [ ] Add per-motor fault state
* [ ] Add total motor power estimate
* [ ] Add top-down quad visualization
* [ ] Animate motor/prop speed
* [ ] Show commanded output separately from resulting RPM

Dashboard example:

```text
M1  ███████████░░░░  63%   18,240 RPM
M2  ███████████░░░░  62%   18,010 RPM
M3  ████████████░░░  66%   19,030 RPM
M4  ███████████░░░░  61%   17,840 RPM
```

---

## Phase 7 — Battery Simulation

* [ ] Add configurable cell count
* [ ] Default MM profile to 6S
* [ ] Add battery capacity
* [ ] Add starting state of charge
* [ ] Add nominal voltage
* [ ] Add per-cell voltage
* [ ] Add total pack voltage
* [ ] Add simulated internal resistance
* [ ] Add voltage sag under load
* [ ] Add current consumption
* [ ] Add consumed mAh
* [ ] Add remaining capacity
* [ ] Add configurable low-voltage thresholds
* [ ] Add forced low-battery condition
* [ ] Add forced cell imbalance
* [ ] Add battery disconnect fault
* [ ] Feed appropriate battery data toward Betaflight where supported

---

## Phase 8 — Basic Flight Physics

The first physics model should prioritize predictable Betaflight interaction rather than perfect aerodynamic realism.

* [ ] Represent quad position
* [ ] Represent linear velocity
* [ ] Represent angular velocity
* [ ] Represent orientation using quaternion
* [ ] Define quad mass
* [ ] Define arm length
* [ ] Define motor thrust coefficient
* [ ] Define motor torque coefficient
* [ ] Convert motor commands into thrust
* [ ] Calculate roll torque
* [ ] Calculate pitch torque
* [ ] Calculate yaw torque
* [ ] Apply gravity
* [ ] Integrate angular motion
* [ ] Integrate linear motion
* [ ] Add basic drag
* [ ] Add ground plane
* [ ] Add reset-to-level function
* [ ] Add reset-to-origin function
* [ ] Make simulation deterministic when configured
* [ ] Separate physics tick rate from GUI refresh rate

Milestone:

```text
Betaflight motor output
        ↓
QuadBench physics
        ↓
virtual gyro / accelerometer
        ↓
Betaflight PID loop
        ↓
new motor output
```

This feedback loop must operate correctly.

---

## Phase 9 — Virtual IMU

* [ ] Generate gyro X/Y/Z
* [ ] Generate accelerometer X/Y/Z
* [ ] Generate attitude
* [ ] Feed IMU data into Betaflight SITL
* [ ] Add configurable sensor noise
* [ ] Add configurable gyro bias
* [ ] Add vibration simulation
* [ ] Add sensor orientation configuration
* [ ] Add incorrect-FC-orientation fault
* [ ] Add frozen gyro fault
* [ ] Add accelerometer calibration fault

---

## Phase 10 — GPS Simulation

* [ ] Define simulated latitude
* [ ] Define simulated longitude
* [ ] Define simulated altitude
* [ ] Define simulated ground speed
* [ ] Define simulated heading
* [ ] Define satellite count
* [ ] Define fix type
* [ ] Add HDOP/accuracy estimate
* [ ] Update GPS state from simulated movement
* [ ] Add GPS enable/disable
* [ ] Add forced loss of fix
* [ ] Add decreasing satellite count
* [ ] Add GPS position drift
* [ ] Add GPS latency
* [ ] Add configurable home position
* [ ] Display distance from home
* [ ] Display direction to home

---

## Phase 11 — Hardware-Like CRSF Receiver

Simple SITL RC injection remains useful, but the long-term receiver implementation should emulate a real serial receiver.

* [ ] Implement CRSF frame encoder
* [ ] Implement RC Channels Packed frames
* [ ] Implement CRSF link statistics
* [ ] Implement RSSI
* [ ] Implement LQ
* [ ] Implement RF mode / packet-rate metadata where appropriate
* [ ] Implement receiver failsafe behavior
* [ ] Add CRSF frame inspector
* [ ] Add deliberately corrupted frames
* [ ] Add dropped-frame simulation
* [ ] Add timing jitter
* [ ] Add serial/UART abstraction
* [ ] Investigate attaching virtual CRSF UART directly to Betaflight SITL

Goal:

```text
RadioMaster Pocket
        ↓ USB HID
QuadBench
        ↓ CRSF
Virtual UART
        ↓
Betaflight SERIAL_RX
```

Betaflight should eventually behave as though a physical CRSF receiver is attached.

---

## Phase 12 — Hardware-Like GPS UART

* [ ] Determine first supported GPS protocol
* [ ] Prefer protocol used by MM if practical
* [ ] Implement GPS serial encoder
* [ ] Implement virtual GPS UART
* [ ] Feed GPS data through serial rather than direct simulator state where possible
* [ ] Support GPS port configuration mistakes
* [ ] Support wrong baud rate
* [ ] Support GPS unplugged
* [ ] Support data present without valid fix
* [ ] Support intermittent GPS connection

Goal:

Betaflight Ports and GPS configuration should matter in the simulator the same way they matter on the real FC.

---

## Phase 13 — Fault Injection

Create explicit fault controls for Betaflight practice.

Receiver faults:

* [ ] RX disconnected
* [ ] RX packet loss
* [ ] RX latency
* [ ] RX jitter
* [ ] Low RSSI
* [ ] Low LQ
* [ ] Corrupt CRSF frames

GPS faults:

* [ ] GPS disconnected
* [ ] No fix
* [ ] Low satellite count
* [ ] GPS drift
* [ ] GPS freeze
* [ ] Bad GPS data

Motor / ESC faults:

* [ ] Motor stalled
* [ ] Motor output limited
* [ ] Motor reversed
* [ ] Motor mapping incorrect
* [ ] ESC offline
* [ ] ESC overheating
* [ ] Motor unable to reach commanded RPM

Sensor faults:

* [ ] Gyro noise
* [ ] Extreme vibration
* [ ] Gyro freeze
* [ ] Incorrect FC orientation
* [ ] Accelerometer bias
* [ ] Accelerometer unavailable

Battery faults:

* [ ] Battery sag
* [ ] Low battery
* [ ] Critical battery
* [ ] Cell imbalance
* [ ] Battery disconnected

---

## Phase 14 — Betaflight Training Scenarios

* [ ] Create scenario system
* [ ] Load scenario from file
* [ ] Reset scenario
* [ ] Add scenario success/failure conditions

Initial scenarios:

* [ ] Receiver configured on wrong UART
* [ ] Serial RX disabled
* [ ] Wrong receiver provider
* [ ] GPS configured on wrong UART
* [ ] GPS present but no fix
* [ ] GPS Rescue unavailable due to insufficient satellites
* [ ] Receiver loss during flight
* [ ] Motor mapping incorrect
* [ ] One motor reversed
* [ ] FC orientation incorrect
* [ ] Excessive gyro vibration
* [ ] Low-battery warning
* [ ] Critical battery
* [ ] Arming disabled
* [ ] Failsafe activation
* [ ] GPS Rescue activation

---

## Phase 15 — Betaflight State / MSP Monitoring

* [ ] Implement MSP connection
* [ ] Read Betaflight status
* [ ] Read arming flags
* [ ] Read active flight modes
* [ ] Read battery information
* [ ] Read receiver information where available
* [ ] Read motor information
* [ ] Read GPS information
* [ ] Read attitude
* [ ] Read sensor status
* [ ] Read firmware/version information
* [ ] Display Betaflight arming-disable reasons
* [ ] Add optional MSP message inspector

QuadBench should observe Betaflight through documented interfaces rather than reaching into internal SITL state wherever practical.

---

## Phase 16 — Logs and Debugging

* [ ] Add application event log
* [ ] Add receiver packet log
* [ ] Add CRSF packet log
* [ ] Add GPS packet log
* [ ] Add SITL packet log
* [ ] Add MSP packet log
* [ ] Add fault-event log
* [ ] Add filter controls
* [ ] Add pause/resume
* [ ] Add clear button
* [ ] Add export-to-file
* [ ] Add selectable debug verbosity
* [ ] Avoid flooding normal logs with high-frequency telemetry

---

## Phase 17 — MM Hardware Profile

Create a hardware profile representing the real 5-inch MM build.

* [ ] Add MM frame configuration
* [ ] Add 6S battery defaults
* [ ] Add MM motor parameters
* [ ] Add MM ESC current limits
* [ ] Add MM motor ordering
* [ ] Add MM prop direction
* [ ] Add MM FC orientation
* [ ] Add MM receiver type
* [ ] Add MM GPS configuration
* [ ] Add MM beeper configuration
* [ ] Add MM UART mapping
* [ ] Store profile independently from simulator defaults

The MM profile should eventually allow practicing configuration against a virtual machine that closely resembles the actual build.

---

## Phase 18 — Persistence

* [ ] Save application configuration
* [ ] Save controller mappings
* [ ] Save quad profiles
* [ ] Save hardware state
* [ ] Save custom scenarios
* [ ] Save UI layout/preferences
* [ ] Add reset-to-defaults
* [ ] Version configuration files for future migrations

---

## Phase 19 — Testing

* [ ] Unit-test CRSF encoding
* [ ] Unit-test receiver channel conversion
* [ ] Unit-test battery model
* [ ] Unit-test motor model
* [ ] Unit-test GPS encoding
* [ ] Unit-test physics integration
* [ ] Unit-test fault injection
* [ ] Add deterministic simulation tests
* [ ] Add Betaflight SITL integration test
* [ ] Test RX loss behavior
* [ ] Test motor mapping
* [ ] Test simulated sensor feedback loop
* [ ] Test reconnect behavior
* [ ] Ensure GUI shutdown cleanly stops worker tasks

---

## Phase 20 — Later / Nice to Have

* [ ] ESC telemetry
* [ ] Bidirectional DShot simulation
* [ ] RPM telemetry
* [ ] RPM filtering interaction
* [ ] ESC configuration profiles
* [ ] Blackbox output capture
* [ ] Blackbox graph integration
* [ ] Virtual barometer
* [ ] Virtual magnetometer
* [ ] Virtual VTX
* [ ] VTX table simulation
* [ ] OSD preview
* [ ] Beeper simulation
* [ ] LED strip simulation
* [ ] Servo outputs
* [ ] Multiple receiver protocols
* [ ] Multiple GPS protocols
* [ ] MAVLink bridge
* [ ] Network remote-control API
* [ ] Record/replay simulations
* [ ] Record/replay receiver input
* [ ] Scenario scripting
* [ ] Headless mode
* [ ] CLI mode
* [ ] Remote dashboard
* [ ] Optional 3D visualization
* [ ] Optional advanced physics backend

---

# First Working Milestone

Do not attempt the full hardware emulation immediately.

Version `0.1.0` should achieve:

* [x] QuadBench GUI launches
* [x] RadioMaster Pocket is detected
* [x] Stick/channel values appear live
* [x] Betaflight SITL connects
* [x] Pocket input reaches Betaflight
* [x] Betaflight motor outputs return to QuadBench
* [ ] Four motor bars move live
* [ ] Basic battery voltage is displayed
* [ ] Basic GPS state is displayed
* [ ] Receiver can be manually disconnected
* [ ] Simulation can be reset without restarting the program

At that point QuadBench becomes useful.

Everything after that incrementally makes the simulated hardware more realistic.
