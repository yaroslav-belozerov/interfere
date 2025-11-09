## Interfere

Interfere is a desktop HTTP client. Minimal, fast, and keyboard-friendly.

### 👍 Features (implemented)
- **GET requests**
  - When sending a request, it's stored along with the response. You can run the same request many times.
- **Keyboard**:
  - Enter: send
  - Ctrl + Enter: send without saving
  - Escape: go back
  - Ctrl + ^v: switch through endpoints
  - Ctrl + <>: switch through responses

### 🗺️ Roadmap
1. Headers in requests  
2. HTTP methods other than GET + request body editing
3. Import/export of data
4. Other protocols and more...

### 💻 Tech stack
- [iced](https://github.com/iced-rs/iced) for GUI
- [reqwest](https://github.com/seanmonstar/reqwest) for HTTP
- [rusqlite](https://github.com/rusqlite/rusqlite) as the SQLite driver

### ▶️ Getting started
> Requires the Rust toolchain.  
> `interfere.db` will be created in the working directory.

Build and run (debug):
```bash
cargo run
```

Build and run (release):
```bash
cargo run --release
```

### 👥 Contributions
Feature requests, bugfixes, and contributions are welcome.  
See this quick guide:
1. Make sure your contribution is  
    - useful
    - unique
    - not AI-generated
2. Create a [Pull Request](https://github.com/yaroslav-belozerov/interfere/compare) in this repo
3. Wait for feedback 😀

### ⚖️ License
The GNU General Public License v3.0. See [LICENSE](https://github.com/yaroslav-belozerov/interfere/blob/main/LICENSE).

