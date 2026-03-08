---
name: skill-creator
description: "Create new SKILL.md skills interactively following the Agent Skills standard"
license: Apache-2.0
metadata:
  author: moltis
  version: "1.0"
allowed-tools: Bash Read Write Edit Glob
---

# Skill Creator

Create new SKILL.md files following the Agent Skills open standard (agentskills.io).

## When to use

Use this skill when the user asks to create a new skill, write a SKILL.md, or make a custom skill.

## Steps

1. Ask the user for:
   - **Name**: kebab-case identifier (e.g., `deploy-vercel`)
   - **Description**: One-line summary of what the skill does
   - **Purpose**: When should the agent use this skill?
2. Create the skill directory and SKILL.md:
   - Personal skills: `~/.moltis/skills/<name>/SKILL.md`
   - Project skills: `.moltis/skills/<name>/SKILL.md`
3. Write the SKILL.md with proper frontmatter:

```markdown
---
name: <name>
description: "<description>"
license: MIT
metadata:
  author: <user>
  version: "1.0"
allowed-tools: <list of tools the skill needs>
---

# <Title>

<Clear instructions for the agent>

## When to use

<Trigger conditions>

## Steps

1. <Step 1>
2. <Step 2>
3. <Step 3>

## Rules

- <Constraint 1>
- <Constraint 2>
```

4. Validate the SKILL.md:
   - `name` in frontmatter matches directory name
   - `description` is present and concise
   - Instructions are clear and actionable
5. Inform the user the skill is ready and how to invoke it (e.g., `/skill-name`).

## Rules

- Always use kebab-case for skill names
- Keep instructions specific and actionable
- Include "When to use" and "Steps" sections at minimum
- List only the tools the skill actually needs in `allowed-tools`
- Default to personal skills directory unless user specifies project scope
