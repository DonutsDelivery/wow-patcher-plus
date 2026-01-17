---
phase: 4-gui-configuration
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src/App.css
  - src/lib/utils.ts
  - src/components/ui/button.tsx
  - src/components/ui/card.tsx
  - src/components/ui/checkbox.tsx
  - src/components/ui/progress.tsx
  - src/components/ui/scroll-area.tsx
  - src/components/ui/select.tsx
  - src/components/ui/switch.tsx
  - vite.config.ts
  - package.json
  - tsconfig.json
  - components.json
autonomous: true

must_haves:
  truths:
    - "Tailwind CSS classes render correctly"
    - "Dark theme is applied by default"
    - "shadcn/ui components are available for use"
  artifacts:
    - path: "src/App.css"
      provides: "Tailwind imports and CSS variables for dark theme"
      contains: "@import \"tailwindcss\""
    - path: "src/lib/utils.ts"
      provides: "cn utility function for class merging"
      exports: ["cn"]
    - path: "src/components/ui/button.tsx"
      provides: "Button component"
      exports: ["Button"]
    - path: "src/components/ui/progress.tsx"
      provides: "Progress bar component"
      exports: ["Progress"]
    - path: "components.json"
      provides: "shadcn/ui configuration"
      contains: "aliases"
  key_links:
    - from: "vite.config.ts"
      to: "@tailwindcss/vite"
      via: "Vite plugin registration"
      pattern: "tailwindcss"
    - from: "src/App.css"
      to: ".dark CSS variables"
      via: "CSS custom properties"
      pattern: "\\.dark"
---

<objective>
Set up Tailwind CSS and shadcn/ui component library with forced dark theme.

Purpose: Establish the UI foundation before building feature components.
Output: Working Tailwind + shadcn/ui setup with dark theme CSS variables and core components.
</objective>

<execution_context>
@./.claude/get-shit-done/workflows/execute-plan.md
@./.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/phases/4-gui-configuration/4-RESEARCH.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Install Tailwind CSS and shadcn/ui dependencies</name>
  <files>package.json, vite.config.ts, tsconfig.json</files>
  <action>
Install required packages:
```bash
npm install tailwindcss @tailwindcss/vite lucide-react class-variance-authority clsx tailwind-merge tw-animate-css @tauri-apps/plugin-store
```

Update vite.config.ts to add Tailwind plugin:
```typescript
import tailwindcss from "@tailwindcss/vite";
// Add to plugins array: tailwindcss()
```

Update tsconfig.json to add path aliases:
```json
{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    }
  }
}
```

Update vite.config.ts to add resolve alias:
```typescript
import path from "path";
// Add to config:
resolve: {
  alias: {
    "@": path.resolve(__dirname, "./src"),
  },
}
```

Initialize shadcn/ui with `npx shadcn@latest init` (select: New York style, Neutral base color, CSS variables yes).
  </action>
  <verify>
`npm run build` completes without errors.
`components.json` exists in project root.
  </verify>
  <done>Tailwind CSS configured with Vite plugin. shadcn/ui initialized with components.json.</done>
</task>

<task type="auto">
  <name>Task 2: Add shadcn/ui components and configure dark theme</name>
  <files>src/App.css, src/lib/utils.ts, src/components/ui/*.tsx</files>
  <action>
Add required shadcn/ui components:
```bash
npx shadcn@latest add button card checkbox progress scroll-area select switch
```

This creates files in src/components/ui/ and src/lib/utils.ts with the cn() function.

Update src/App.css to include Tailwind and dark theme CSS variables (from 4-RESEARCH.md):
- Import tailwindcss
- Import tw-animate-css
- Add @custom-variant dark rule
- Add :root light theme variables
- Add .dark dark theme variables with OKLCH colors

The CSS should match the exact format from 4-RESEARCH.md Code Examples section.
  </action>
  <verify>
Files exist:
- `src/lib/utils.ts`
- `src/components/ui/button.tsx`
- `src/components/ui/progress.tsx`
- `src/components/ui/card.tsx`

`npm run build` completes without CSS errors.
  </verify>
  <done>shadcn/ui components installed. Dark theme CSS variables configured in App.css.</done>
</task>

<task type="auto">
  <name>Task 3: Enable forced dark mode and verify setup</name>
  <files>src/main.tsx, src/App.tsx</files>
  <action>
Update src/main.tsx to force dark mode on app load:
```typescript
// Add before ReactDOM.createRoot:
document.documentElement.classList.add('dark');
```

Update src/App.tsx to use a shadcn/ui component as verification:
```typescript
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import "./App.css";

function App() {
  return (
    <div className="min-h-screen bg-background text-foreground p-8">
      <h1 className="text-2xl font-bold mb-4">Turtle WoW HD Patcher</h1>
      <p className="text-muted-foreground mb-4">Setting up UI...</p>
      <Progress value={33} className="w-64 mb-4" />
      <Button>Test Button</Button>
    </div>
  );
}

export default App;
```

This serves as a smoke test that:
1. Dark theme CSS variables work (bg-background should be dark)
2. Tailwind classes compile (min-h-screen, p-8, etc.)
3. shadcn/ui components render (Button, Progress)
  </action>
  <verify>
`npm run tauri dev` launches app with:
- Dark background (not white)
- Styled button visible
- Progress bar visible at 33%
  </verify>
  <done>Dark theme forced on load. Tailwind + shadcn/ui rendering correctly in Tauri app.</done>
</task>

</tasks>

<verification>
1. `npm run build` completes without errors
2. `npm run tauri dev` shows dark-themed app with Button and Progress components
3. All src/components/ui/*.tsx files exist and export their components
4. src/lib/utils.ts exports cn function
</verification>

<success_criteria>
- Tailwind CSS classes compile and render
- Dark theme applied automatically (no light flash)
- shadcn/ui Button, Card, Checkbox, Progress, ScrollArea, Select, Switch components available
- Path alias @/* resolves correctly
</success_criteria>

<output>
After completion, create `.planning/phases/4-gui-configuration/4-01-SUMMARY.md`
</output>
