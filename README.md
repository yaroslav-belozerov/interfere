## Interfere

Interfere is a desktop HTTP client. Minimal, fast, and keyboard-friendly.

### ▶️ Running 
> Available for Linux, Windows, MacOS.
> `interfere.db` will be created in the working directory.

[🔗 Get the executable for your OS](https://github.com/yaroslav-belozerov/interfere/releases)  

### 👍 Features (implemented)
- **GET, POST requests**
  - Send request, save the response.
  - You can override a saved response, or send request in draft mode. 
  - Query params & headers are working.
- **Keymap**:
  - Enter: send
  - Ctrl + Enter: send without saving
  - Escape: go back
  - Ctrl + ^v: switch through endpoints
  - Ctrl + <>: switch through responses

### 🗺️ Roadmap
1. HTTP methods other than GET/POST + request body editing
2. Import/export of data
3. Better UI/UX
4. Other protocols and more...

### 💻 Tech stack
- [iced](https://github.com/iced-rs/iced) for GUI
- [reqwest](https://github.com/seanmonstar/reqwest) for HTTP
- [rusqlite](https://github.com/rusqlite/rusqlite) as the SQLite driver

### 🧑‍💻 Build yourself 
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

## 🖌️ Screenshots 
<div style="display: flex; flex-wrap: wrap;">
<img alt="Interfere Banner" src="/misc/screenshots/plain.png" style="width: 30%" />
<img alt="Interfere Banner" src="/misc/screenshots/format.png" style="width: 30%" />
<img alt="Interfere Banner" src="/misc/screenshots/auto_query.png" style="width: 30%" />
</div>

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

