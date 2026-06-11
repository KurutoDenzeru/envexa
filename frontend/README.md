# Envexa Web Dashboard (Frontend)

This directory contains the web dashboard for Envexa. It is a modern React application built on **TanStack Start**, styled with **TailwindCSS**, and uses **Shadcn UI** for accessible, premium components.

## Tech Stack
- **Framework**: [TanStack Start](https://tanstack.com/start)
- **UI Components**: [Shadcn UI](https://ui.shadcn.com/)
- **Icons**: [Lucide React](https://lucide.dev/)
- **Styling**: Tailwind CSS
- **Package Manager**: [Bun](https://bun.sh/)

## Development

To start the development server:

```bash
bun install
bun run dev
```

The application will be available at `http://localhost:3000`.

## Architecture
- `src/routes/`: Contains TanStack Start file-based routing components.
- `src/components/ui/`: Contains Shadcn UI components.
- `src/styles.css`: Global Tailwind CSS configuration and theme variables.

## Building for Production
The frontend is compiled and bundled directly into the `envexa` Rust executable using `rust-embed`. The build output is placed in `.output/public/` (or similar depending on TanStack Start's static adapter).

To build the static assets:
```bash
bun run build
```
