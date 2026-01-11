# SQL Tauri

A modern, cross-platform SQL database client built with Tauri, SvelteKit, and Rust. Manage your PostgreSQL and MySQL databases with a beautiful, intuitive interface.

![Tauri](https://img.shields.io/badge/Tauri-2.0-blue?logo=tauri)
![Svelte](https://img.shields.io/badge/Svelte-5.0-orange?logo=svelte)
![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)
![License](https://img.shields.io/badge/License-MIT-green)

## Features

### Database Support
- **PostgreSQL** - Full support for PostgreSQL databases
- **MySQL** - Full support for MySQL databases

### Connection Management
- Save, edit, and manage multiple database connections
- Quick connect to saved connections
- Persistent connection storage

### SSH Tunnel Support
- Connect to remote databases through SSH tunnels
- Support for password and key-based authentication
- Automatic SSH config file (`~/.ssh/config`) parsing
- Tunnel connection reuse for efficiency

### Data Browsing
- Browse databases and tables in a tree view
- View table data with pagination
- Filter data with flexible operators (equals, contains, greater than, etc.)
- Sort by any column (ascending/descending)
- View total row counts

### Table Structure
- View column definitions (name, type, nullable, defaults)
- View column comments
- View foreign key relationships
- View table indexes (primary, unique, btree, hash, etc.)

### SQL Editor
- Built-in SQL editor with syntax highlighting (CodeMirror)
- Execute custom SQL queries
- View query results in a formatted table

### Modern UI
- Clean, modern interface built with Tailwind CSS
- Tab-based workflow for multiple tables/queries
- Keyboard shortcuts (Cmd/Ctrl+W to close tabs)
- Dark mode support
- Responsive resizable panels

## Tech Stack

### Frontend
- **SvelteKit 5** - Modern reactive UI framework
- **TypeScript** - Type-safe JavaScript
- **Tailwind CSS 4** - Utility-first CSS framework
- **CodeMirror 6** - Extensible code editor
- **bits-ui** - Headless UI components
- **Lucide** - Beautiful icons

### Backend
- **Tauri 2** - Rust-based desktop app framework
- **SQLx** - Async SQL toolkit for Rust
- **ssh2** - SSH2 client library for tunneling
- **Tokio** - Async runtime

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (v18 or later)
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Bun](https://bun.sh/) (recommended) or npm/yarn

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/sql-tauri.git
   cd sql-tauri
   ```

2. Install frontend dependencies:
   ```bash
   bun install
   # or
   npm install
   ```

3. Run in development mode:
   ```bash
   bun tauri dev
   # or
   npm run tauri dev
   ```

4. Build for production:
   ```bash
   bun tauri build
   # or
   npm run tauri build
   ```

## Usage

### Creating a Connection

1. Click the **+** button in the sidebar to create a new connection
2. Fill in the connection details:
   - **Connection Name** - A friendly name for your connection
   - **Database Type** - PostgreSQL or MySQL
   - **Host** - Database server hostname
   - **Port** - Database port (5432 for PostgreSQL, 3306 for MySQL)
   - **Username** - Database username
   - **Password** - Database password
   - **Database** - Database name to connect to
3. (Optional) Enable SSH Tunnel for remote connections:
   - **SSH Host** - SSH server hostname
   - **SSH Port** - SSH port (default: 22)
   - **SSH User** - SSH username
   - **SSH Password** or **SSH Key Path** - Authentication method
4. Click **Save** to save the connection, or **Connect** to connect immediately

### Browsing Data

1. Connect to a database from the sidebar
2. Expand the table list to see available tables
3. Click on a table name to open it in a new tab
4. Use the toolbar to:
   - Add filters to narrow down results
   - Sort by clicking column headers
   - Navigate pages for large datasets
5. Right-click on a table for more options (view structure, etc.)

### Running Queries

1. Click the SQL terminal icon to open a new query tab
2. Write your SQL query in the editor
3. Execute the query to see results

## Project Structure

```
sql-tauri/
├── src/                    # SvelteKit frontend
│   ├── lib/
│   │   ├── components/     # UI components
│   │   ├── partials/       # Layout partials
│   │   ├── stores/         # Svelte stores for state management
│   │   ├── db.ts           # Database API bindings
│   │   └── utils.ts        # Utility functions
│   └── routes/             # SvelteKit routes/pages
├── src-tauri/              # Tauri/Rust backend
│   └── src/
│       ├── connection_manager.rs  # Save/load connections
│       ├── database_provider.rs   # Database provider trait
│       ├── db.rs                  # Database commands
│       ├── mysql_provider.rs      # MySQL implementation
│       ├── postgres_provider.rs   # PostgreSQL implementation
│       ├── ssh_tunnel.rs          # SSH tunnel management
│       └── state.rs               # App state persistence
├── static/                 # Static assets
└── package.json
```

## Development

### Available Scripts

```bash
# Start development server
bun dev

# Run Tauri in development mode
bun tauri dev

# Build for production
bun tauri build

# Type check
bun check
```

### IDE Setup

Recommended extensions for VS Code:
- [Svelte for VS Code](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode)
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
