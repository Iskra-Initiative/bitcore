# bitcore

`bitcore` is generic. It provides an API to a serial driver implementation, which can be used for any project, which requires serial communication.

- **interface**
  - provides a public API to a serial driver implementation
  - check `src/api.rs` for details

- **core**
  - thread-safe serial port operations
  - multiple connection support
  - operation retry mechanism
  - timeouts
  - debug trace
  - *TODO*: report/statistics generation

- **modular**
  - easily extendable standard implementation
  - allows for tailoring to potential non-supported devices

- **performance**
  - configurable
  - *TODO*: optimization

- **security**
  - *--*

- **TODO**
  - admin:
    - add license
    - add documentation (Vocs / Nextra / GitBook)
  - tech:
    - buffered write/read
    - connection validation ('heart beat')
    - extend api with an action loop default implementation (callable by UI)
