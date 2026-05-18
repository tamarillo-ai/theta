---
name: create-section
description: Use when creating a new section for the AI Workshop presentation. Guides through title slide setup, layout choices, and content gathering. Activates when user wants to add a new topic section to the slideshow.
---

# Create New Section Skill

This skill guides the creation of a new section for the AI Native Developer Workshop presentation.

## When to Activate

- User asks to "create a new section"
- User wants to "add slides about [topic]"
- User mentions adding a new topic to the workshop
- User requests a new section file in `src/sections/`

## Workflow

### Step 1: Gather Section Requirements

Before creating any files, **ask the user** these questions:

1. **Section Topic**: What is the main topic for this section?
2. **Section Color**: Which color fits best?
   - `blue`
   - `green`
   - `purple`
   - `orange`
   - `gray` - Overview/Introduction topics
3. **Source Material**: Does the user have a URL or reference to pull content from?

### Step 2: Content Gathering (Optional)

If the user provides a URL or wants help researching content:

**Invoke the Slide Content Creator agent** using:

```text
@agent Slide Content Creator [URL or topic]
```

The Slide Content Creator will:

- Fetch content from the provided URL
- Extract verified facts only
- Propose a structured list of slides
- Validate all claims against sources

Wait for the agent's output before proceeding to implementation.

### Step 3: Title Slide Setup

Every section MUST begin with a title slide. Use this template:

```tsx
import { IconName } from 'lucide-react';
import { SlideType } from './types';

export const {sectionName}Slides: SlideType[] = [
  {
    title: "",
    subtitle: "",
    content: (
      <div className="flex flex-col items-center justify-center h-full space-y-6">
        <IconName className="w-16 h-16 md:w-20 md:h-20 text-{color}-500" />
        <h1 className="text-5xl md:text-6xl font-bold text-{color}-900 text-center">
          Section Name
        </h1>
        <p className="text-xl md:text-2xl text-gray-600 text-center max-w-2xl">
          Brief tagline describing what this section covers
        </p>
        <div className="flex space-x-2 mt-4">
          <div className="w-3 h-3 bg-{color}-300 rounded-full"></div>
          <div className="w-3 h-3 bg-{color}-500 rounded-full"></div>
          <div className="w-3 h-3 bg-{color}-300 rounded-full"></div>
        </div>
      </div>
    )
  },
  // Content slides follow...
];
```

### Recommended Icons by Topic

| Topic | Suggested Icons |
| ----- | --------------- |
| LLMs/AI | `Sparkles`, `Brain`, `Cpu` |
| Models | `Box`, `Layers` |
| Framework | `Target`, `Compass` |
| Code/Copilot | `Code`, `Bot` |
| Instructions | `ScrollText`, `FileText` |
| Skills | `Puzzle`, `Wrench` |
| Spaces/Organization | `Layout`, `FolderOpen` |
| Windows/UI | `Monitor`, `PanelLeft` |
| Multi-Agent | `Network`, `Users` |
| Workflows | `Repeat`, `RefreshCw` |
| Context | `Brain`, `Database` |
| Security | `Shield`, `Lock` |
| Privacy | `Eye`, `EyeOff` |
| Testing | `TestTube`, `CheckCircle` |

### Step 4: File Creation Checklist

When creating the section, complete ALL of these:

1. **Create section file**: `src/sections/{sectionname}.tsx`
   - Include title slide as first slide
   - Export slides array: `export const {sectionName}Slides: SlideType[] = [...]`

2. **Update index.ts**: Add to `src/sections/index.ts`

   ```tsx
   export { {sectionName}Slides } from './{sectionname}';
   ```

3. **Update main.tsx imports**: Add to import statement

   ```tsx
   import { ..., {sectionName}Slides } from './sections';
   ```

4. **Add to sections array** in `src/main.tsx`:

   ```tsx
   { name: 'Section Name', slides: {sectionName}Slides, color: '{color}', icon: IconName },
   ```

5. **Add to slides spread** in main.tsx:

   ```tsx
   const slides = [...otherSlides, ...{sectionName}Slides];
   ```

6. **Import icon** at top of main.tsx if not already imported

### Step 5: Content Slide Patterns

After the title slide, add content slides using these patterns:

**Definition Slide** (introducing concepts):

```tsx
{
  title: "Concept Name",
  subtitle: "Clear context sentence",
  content: (
    <div className="flex flex-col space-y-6 max-w-3xl mx-auto">
      <div className="bg-{color}-50 p-6 rounded-lg border-l-4 border-{color}-500">
        <h3 className="text-2xl font-bold text-{color}-900 mb-4">What is [Concept]?</h3>
        <p className="text-lg text-gray-700">Definition here.</p>
      </div>
      {/* Key points, takeaway */}
    </div>
  )
}
```

**Grid Slide** (multiple related items):

```tsx
{
  title: "Overview Title",
  subtitle: "Context",
  content: (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-4 md:gap-6 max-w-3xl mx-auto">
      {/* 2-4 cards */}
    </div>
  )
}
```

**List Slide** (step-by-step content):

```tsx
{
  title: "Process/Steps",
  subtitle: "Context",
  content: (
    <div className="flex flex-col space-y-6 max-w-3xl mx-auto">
      {/* Header box, bullet lists, takeaway */}
    </div>
  )
}
```

## Validation Checklist

Before completing, verify:

- [ ] Title slide has empty `title` and `subtitle` strings
- [ ] Icon imported from `lucide-react`
- [ ] Color scheme is consistent throughout section
- [ ] All 4 files updated (section file, index.ts, main.tsx imports, main.tsx arrays)
- [ ] TypeScript compiles without errors
- [ ] Responsive classes used (`md:` breakpoints where appropriate)

## Example Conversation Flow

**User**: "Create a new section about prompt engineering"

**Response**:
"I'd like to help you create a new section about prompt engineering. A few questions first:

1. **Color scheme**: This seems like it fits **green** - helping users communicate effectively with AI. Does that work, or would you prefer a different color?

2. **Source material**: Do you have any URLs or documentation you'd like me to pull content from? I can use the Slide Content Creator agent to extract verified facts from sources.

3. **Scope**: What key topics should this section cover? (e.g., prompt structure, few-shot examples, chain-of-thought, etc.)

Once I have this info, I'll set up the title slide and content structure."
