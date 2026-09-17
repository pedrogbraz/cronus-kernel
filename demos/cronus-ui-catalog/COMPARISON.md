# React vs `.cronus`

Every documented example of the cronus-ui React catalog next to the `.cronus`
that renders the same component in the kernel (`demos/cronus-ui-catalog/<family>.cronus`,
served at `/<family>`). Left: the JSX from the docs page. Right: the declaration.
No imports, no JSX: the kernel emits the HTML, tokens and motion.


194 families · 292 examples.

## Buttons

### Button (`/button`)

#### Variants

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex flex-wrap items-center gap-3">
  <Button variant="primary">Primary</Button>
  <Button variant="secondary">Secondary</Button>
  <Button variant="outline">Outline</Button>
  <Button variant="ghost">Ghost</Button>
  <Button variant="destructive">Destructive</Button>
  <Button variant="link">Link</Button>
</div>
```

</td><td>

```cronus
component Primary layout:inline style:button+primary { label "Primary" }
component Secondary layout:inline style:button+secondary { label "Secondary" }
component Outline layout:inline style:button+outline { label "Outline" }
component Ghost layout:inline style:button+ghost { label "Ghost" }
component Destructive layout:inline style:button+destructive { label "Destructive" }
component Link layout:inline style:button+link { label "Link" }
```

</td></tr></table>

#### Sizes

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex flex-wrap items-center gap-3">
  <Button size="sm">Small</Button>
  <Button size="md">Medium</Button>
  <Button size="lg">Large</Button>
  <Button size="icon" aria-label="Settings">
    <Settings />
  </Button>
  <Button size="icon-sm" aria-label="Settings">
    <Settings />
  </Button>
</div>
```

</td><td>

```cronus
component Small layout:inline style:button+sm { label "Small" }
component Medium layout:inline style:button+md { label "Medium" }
component Large layout:inline style:button+lg { label "Large" }
component Icon layout:inline style:button+icon icon:settings { label "Settings" }
component IconSmall layout:inline style:button+icon-sm icon:settings { label "Settings" }
```

</td></tr></table>

#### With icons

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex flex-wrap items-center gap-3">
  <Button>
    <Download />
    Download
  </Button>
  <Button variant="secondary">
    Continue
    <ArrowRight />
  </Button>
  <Button variant="outline">
    <Heart />
    Like
  </Button>
  <Button disabled>
    <Heart />
    Disabled
  </Button>
  <Button disabled>
    <Spinner size="sm" aria-hidden="true" />
    Saving
  </Button>
</div>
```

</td><td>

```cronus
component Download layout:inline style:button icon:download { label "Download" }
component Continue layout:inline style:button+secondary icon-end:arrow-right { label "Continue" }
component Like layout:inline style:button+outline icon:heart { label "Like" }
component Disabled layout:inline style:button icon:heart disabled:true { label "Disabled" }
component Saving layout:inline style:button loading:true disabled:true { label "Saving" }
```

</td></tr></table>

#### As link

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Button asChild>
  <a href="/docs">
    Read the docs
    <ArrowRight />
  </a>
</Button>
```

</td><td>

```cronus
component ReadDocs layout:inline style:button icon-end:arrow-right { label "Read the docs" -> "/docs" }
```

</td></tr></table>

### AnimatedButton (`/animated-button`)

#### Spring feedback

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex flex-wrap items-center gap-3">
  <AnimatedButton variant="primary">Primary</AnimatedButton>
  <AnimatedButton variant="outline">Outline</AnimatedButton>
  <AnimatedButton variant="secondary">
    <Download />
    Download
  </AnimatedButton>
</div>
```

</td><td>

```cronus
component SpringPrimary layout:inline style:animated-button+primary { label "Primary" }
component SpringOutline layout:inline style:animated-button+outline { label "Outline" }
component SpringDownload layout:inline style:animated-button+secondary icon:download { label "Download" }
```

</td></tr></table>

### Toggle (`/toggle`)

#### Default

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const [bold, setBold] = useState(false);

return (
  <Toggle pressed={bold} onPressedChange={setBold} aria-label="Toggle bold">
    <Bold />
    Bold
  </Toggle>
);
```

</td><td>

```cronus
component Bold layout:inline style:toggle icon:bold aria-label:"Toggle bold" { label "Bold" }
```

</td></tr></table>

#### Outline variant

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const [italic, setItalic] = useState(false);

return (
  <Toggle
    variant="outline"
    pressed={italic}
    onPressedChange={setItalic}
    aria-label="Toggle italic"
  >
    <Italic />
    Italic
  </Toggle>
);
```

</td><td>

```cronus
component Italic layout:inline style:toggle+outline icon:italic aria-label:"Toggle italic" { label "Italic" }
```

</td></tr></table>

#### Sizes

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const [pressed, setPressed] = useState(true);

return (
  <div className="flex items-center gap-3">
    <Toggle size="sm" pressed={pressed} onPressedChange={setPressed} aria-label="Bold (small)">
      <Bold />
    </Toggle>
    <Toggle size="md" pressed={pressed} onPressedChange={setPressed} aria-label="Bold (medium)">
      <Bold />
    </Toggle>
    <Toggle size="lg" pressed={pressed} onPressedChange={setPressed} aria-label="Bold (large)">
      <Bold />
    </Toggle>
  </div>
);
```

</td><td>

```cronus
component BoldSmall layout:inline style:toggle+sm icon:bold icon-only:true pressed:true aria-label:"Bold (small)" { label "Bold" }
component BoldMedium layout:inline style:toggle+md icon:bold icon-only:true pressed:true aria-label:"Bold (medium)" { label "Bold" }
component BoldLarge layout:inline style:toggle+lg icon:bold icon-only:true pressed:true aria-label:"Bold (large)" { label "Bold" }
```

</td></tr></table>

### ToggleGroup (`/toggle-group`)

#### Single

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const [align, setAlign] = useState("left");

return (
  <ToggleGroup
    type="single"
    value={align}
    onValueChange={(value) => {
      if (value) setAlign(value);
    }}
  >
    <ToggleGroupItem value="left" aria-label="Align left">
      <AlignLeft />
    </ToggleGroupItem>
    <ToggleGroupItem value="center" aria-label="Align center">
      <AlignCenter />
    </ToggleGroupItem>
    <ToggleGroupItem value="right" aria-label="Align right">
      <AlignRight />
    </ToggleGroupItem>
  </ToggleGroup>
);
```

</td><td>

```cronus
component Align layout:inline style:toggle-group value:"Align left" icon-only:true {
  item "Align left" icon:align-left
  item "Align center" icon:align-center
  item "Align right" icon:align-right
}
```

</td></tr></table>

#### Multiple

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const [formats, setFormats] = useState<string[]>(["bold"]);

return (
  <ToggleGroup type="multiple" value={formats} onValueChange={setFormats}>
    <ToggleGroupItem value="bold" aria-label="Bold">
      <Bold />
    </ToggleGroupItem>
    <ToggleGroupItem value="italic" aria-label="Italic">
      <Italic />
    </ToggleGroupItem>
    <ToggleGroupItem value="underline" aria-label="Underline">
      <Underline />
    </ToggleGroupItem>
  </ToggleGroup>
);
```

</td><td>

```cronus
component Formats layout:inline style:toggle-group type:multiple value:"Bold" icon-only:true {
  item "Bold" icon:bold
  item "Italic" icon:italic
  item "Underline" icon:underline
}
```

</td></tr></table>

### CopyButton (`/copy-button`)

#### Default

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<CopyButton value="npm install @cronus-ui/ui" />
```

</td><td>

```cronus
component CopyInstall layout:inline style:copy-button value:"npm install @cronus-ui/ui" { label "Copy" }
```

</td></tr></table>

#### Inline with a value

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex items-center gap-2">
  <code className="rounded bg-surface-inset px-2 py-1 font-mono text-sm">cronus_sk_live_…a1b2</code>
  <CopyButton
    value="cronus_sk_live_a1b2"
    variant="outline"
    copyLabel="Copy API key"
    copiedLabel="API key copied"
  />
</div>
```

</td><td>

```cronus
component CopyKey layout:inline style:copy-button+outline value:"cronus_sk_live_a1b2" aria-label:"Copy API key" { label "Copy API key" }
```

</td></tr></table>

### ButtonGroup (`/button-group`)

#### Horizontal

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<ButtonGroup>
  <Button variant="outline">
    <ChevronLeft />
    Prev
  </Button>
  <Button variant="outline">Page 1</Button>
  <Button variant="outline">
    Next
    <ChevronRight />
  </Button>
</ButtonGroup>
```

</td><td>

```cronus
component Pager layout:inline style:button-group+outline aria-label:"Pages" {
  item "Prev" icon:chevron-left
  item "Page 1"
  item "Next" icon-end:chevron-right
}
```

</td></tr></table>

#### Vertical

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<ButtonGroup orientation="vertical">
  <Button variant="outline">
    <AlignLeft />
    Left
  </Button>
  <Button variant="outline">
    <AlignCenter />
    Center
  </Button>
  <Button variant="outline">
    <AlignRight />
    Right
  </Button>
</ButtonGroup>
```

</td><td>

```cronus
component Alignment layout:inline style:button-group+outline+vertical aria-label:"Alignment" {
  item "Left" icon:align-left
  item "Center" icon:align-center
  item "Right" icon:align-right
}
```

</td></tr></table>

### Fab (`/fab`)

#### Default

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="relative h-64 rounded-xl border border-border">
  <Fab
    icon={<Plus />}
    label="New item"
    className="absolute bottom-4 right-4"
  />
</div>
```

</td><td>

```cronus
component NewItem layout:inline style:fab icon:plus { label "New item" }
```

</td></tr></table>

#### Speed dial

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="relative h-64 rounded-xl border border-border">
  <Fab
    icon={<Plus />}
    label="Create"
    className="absolute bottom-4 right-4"
    actions={[
      { icon: <Pencil />, label: "Write a note" },
      { icon: <Image />, label: "Upload an image" },
      { icon: <Upload />, label: "Import a file" },
    ]}
  />
</div>
```

</td><td>

```cronus
component Create layout:inline style:fab icon:plus {
  label "Create"
  action "Write a note" icon:pencil
  action "Upload an image" icon:image
  action "Import a file" icon:upload
}
```

</td></tr></table>

### SplitButton (`/split-button`)

#### Default

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const [status, setStatus] = useState("Draft saved 2 minutes ago");

return (
  <div className="flex flex-col items-start gap-3">
    <SplitButton
      icon={<Save />}
      onClick={() => setStatus("Saved just now")}
      items={[
        { icon: <FileText />, label: "Save as draft", onSelect: () => setStatus("Saved to drafts") },
        { icon: <Copy />, label: "Save a copy", onSelect: () => setStatus("Copy saved") },
        {
          icon: <Trash2 />,
          label: "Discard changes",
          destructive: true,
          onSelect: () => setStatus("Changes discarded"),
        },
      ]}
    >
      Save
    </SplitButton>
    <p className="text-sm text-fg-tertiary">{status}</p>
  </div>
);
```

</td><td>

```cronus
component Save layout:inline style:split-button icon:save {
  label "Save"
  item "Save as draft" icon:file-text
  item "Save a copy" icon:copy
  item "Discard changes" icon:trash-2 tone:danger
}
```

</td></tr></table>

#### Variants

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex flex-wrap items-center gap-3">
  <SplitButton
    variant="primary"
    items={[
      { icon: <Copy />, label: "Duplicate" },
      { icon: <Share2 />, label: "Share" },
    ]}
  >
    Primary
  </SplitButton>
  <SplitButton
    variant="secondary"
    items={[
      { icon: <Copy />, label: "Duplicate" },
      { icon: <Share2 />, label: "Share" },
    ]}
  >
    Secondary
  </SplitButton>
  <SplitButton
    variant="outline"
    items={[
      { icon: <Copy />, label: "Duplicate" },
      { icon: <Share2 />, label: "Share" },
    ]}
  >
    Outline
  </SplitButton>
</div>
```

</td><td>

```cronus
component SplitPrimary layout:inline style:split-button+primary {
  label "Primary"
  item "Duplicate" icon:copy
  item "Share" icon:share-2
}
component SplitSecondary layout:inline style:split-button+secondary {
  label "Secondary"
  item "Duplicate" icon:copy
  item "Share" icon:share-2
}
component SplitOutline layout:inline style:split-button+outline {
  label "Outline"
  item "Duplicate" icon:copy
  item "Share" icon:share-2
}
```

</td></tr></table>

#### Loading

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const [loading, setLoading] = useState(false);
const [published, setPublished] = useState(false);

const publish = () => {
  setLoading(true);
  setPublished(false);
  setTimeout(() => {
    setLoading(false);
    setPublished(true);
  }, 1400);
};

return (
  <div className="flex flex-col items-start gap-3">
    <SplitButton
      icon={<Send />}
      loading={loading}
      onClick={publish}
      menuLabel="Publishing options"
      items={[
        { icon: <CalendarClock />, label: "Schedule for later" },
        { icon: <FileText />, label: "Save as draft" },
      ]}
    >
      {loading ? "Publishing…" : "Publish"}
    </SplitButton>
    <p className="text-sm text-fg-tertiary">
      {published ? "Your post is live." : "Ready when you are."}
    </p>
  </div>
);
```

</td><td>

```cronus
component Publish layout:inline style:split-button icon:send loading:true menuLabel:"Publishing options" {
  label "Publishing…"
  item "Schedule for later" icon:calendar-clock
  item "Save as draft" icon:file-text
}
```

</td></tr></table>

### ModeToggle (`/mode-toggle`)

#### Sun ⇄ moon morph

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const [mode, setMode] = useState<"light" | "dark">("light");

return <ModeToggle mode={mode} onModeChange={setMode} />;
```

</td><td>

```cronus
component Morph layout:inline style:mode-toggle+light { label "Mode" }
```

</td></tr></table>

#### Sizes

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const [mode, setMode] = useState<"light" | "dark">("dark");

return (
  <div className="flex items-center gap-3">
    <ModeToggle size="sm" mode={mode} onModeChange={setMode} />
    <ModeToggle size="md" mode={mode} onModeChange={setMode} />
  </div>
);
```

</td><td>

```cronus
component MorphSmall layout:inline style:mode-toggle+dark+sm { label "Mode" }
component MorphMedium layout:inline style:mode-toggle+dark+md { label "Mode" }
```

</td></tr></table>

#### Wired to a theme provider

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const { mode, setMode } = useTheme();

return <ModeToggle mode={mode} onModeChange={setMode} />;
```

</td><td>

```cronus
(see mode-toggle.cronus)
```

</td></tr></table>

## Forms

### Input (`/input`)

#### Default

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Input placeholder="you@cronus.dev" />
```

</td><td>

```cronus
component Email layout:stack style:input { text "you@cronus.dev" }
```

</td></tr></table>

#### Invalid state

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Field>
  <FieldLabel htmlFor="email">Email</FieldLabel>
  <Input
    id="email"
    invalid
    defaultValue="not-an-email"
    aria-describedby="email-err"
  />
  <FieldError id="email-err">Enter a valid email address.</FieldError>
</Field>
```

</td><td>

```cronus
component EmailInvalid layout:stack style:field control:input type:email value:"not-an-email" error:"Enter a valid email address." { label "Email" }
```

</td></tr></table>

#### Disabled

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Input placeholder="Disabled" disabled />
```

</td><td>

```cronus
component DisabledInput layout:stack style:input disabled:true { text "Disabled" }
```

</td></tr></table>

### InputGroup (`/input-group`)

#### Prefix & suffix

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<InputGroup>
  <InputGroupAddon>
    <Link2 />
    https://
  </InputGroupAddon>
  <Input placeholder="acme.cronus.app" />
  <InputGroupAddon align="end">.cronus.app</InputGroupAddon>
</InputGroup>
```

</td><td>

```cronus
component Domain layout:stack style:input-group addon:"https://" addon-icon:link-2 addon-end:".cronus.app" { text "acme.cronus.app" }
```

</td></tr></table>

#### Leading icon

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<InputGroup>
  <InputGroupAddon>
    <AtSign />
  </InputGroupAddon>
  <Input type="email" placeholder="you@company.com" />
</InputGroup>
```

</td><td>

```cronus
component EmailAt layout:stack style:input-group addon-icon:at-sign type:email { text "you@company.com" }
```

</td></tr></table>

### PasswordInput (`/password-input`)

#### Default

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<PasswordInput
  className="max-w-sm"
  placeholder="Enter your password"
  autoComplete="current-password"
/>
```

</td><td>

```cronus
component Current layout:stack style:password-input { text "Enter your password" }
```

</td></tr></table>

#### Strength meter

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function PasswordInputStrengthDemo() {
  const [value, setValue] = useState("");

  return (
    <div className="w-full max-w-sm">
      <PasswordInput
        value={value}
        onChange={(event) => setValue(event.target.value)}
        showStrength
        placeholder="Create a password"
        autoComplete="new-password"
      />
    </div>
  );
}
```

</td><td>

```cronus
component Strength layout:stack style:password-input strength:true { text "Create a password" }
```

</td></tr></table>

### Textarea (`/textarea`)

#### Default

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Textarea placeholder="Tell us what you're building…" rows={4} />
```

</td><td>

```cronus
component Building layout:stack style:textarea rows:4 { text "Tell us what you're building…" }
```

</td></tr></table>

#### Invalid

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Field>
  <FieldLabel htmlFor="bio">Bio</FieldLabel>
  <Textarea
    id="bio"
    invalid
    rows={4}
    defaultValue="…"
    aria-describedby="bio-err"
  />
  <FieldError id="bio-err">This value isn't allowed.</FieldError>
</Field>
```

</td><td>

```cronus
component BioInvalid layout:stack style:field control:textarea rows:4 value:"…" error:"This value isn't allowed." { label "Bio" }
```

</td></tr></table>

### Label (`/label`)

#### With input

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex flex-col gap-1.5">
  <Label htmlFor="workspace">Workspace name</Label>
  <Input id="workspace" placeholder="acme-inc" />
</div>
```

</td><td>

```cronus
component WorkspaceLabel layout:stack style:label for:workspace { label "Workspace name" }
component WorkspaceInput layout:stack style:input id:workspace { text "acme-inc" }
```

</td></tr></table>

### Checkbox (`/checkbox`)

#### Default

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function CheckboxDemo() {
  const [checked, setChecked] = useState(true);

  return (
    <div className="flex items-center gap-3">
      <Checkbox
        id="terms"
        checked={checked}
        onCheckedChange={(value) => setChecked(value === true)}
      />
      <Label htmlFor="terms">Accept terms &amp; conditions</Label>
    </div>
  );
}
```

</td><td>

```cronus
component Unavailable layout:inline style:checkbox disabled:true { label "Unavailable option" text "Unavailable option" }
component Terms layout:inline style:checkbox checked:true { label "Accept the terms" text "Accept the terms" }
```

</td></tr></table>

#### Disabled

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex items-center gap-3">
  <Checkbox id="disabled" disabled />
  <Label htmlFor="disabled">Unavailable option</Label>
</div>
```

</td><td>

```cronus
(see checkbox.cronus)
```

</td></tr></table>

### AnimatedCheckbox (`/animated-checkbox`)

#### Default

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function AnimatedCheckboxDemo() {
  return (
    <div className="flex flex-col items-start gap-3">
      <AnimatedCheckbox title="Implement Checkbox" />
      <AnimatedCheckbox title="Write documentation" />
      <AnimatedCheckbox title="Add tests" defaultChecked />
    </div>
  );
}
```

</td><td>

```cronus
component Ship layout:inline style:animated-checkbox title:"Ship the release" checked:true { label "Ship the release" }
component Implement layout:inline style:animated-checkbox title:"Implement Checkbox" { label "Implement Checkbox" }
component UnavailableRow layout:inline style:animated-checkbox title:"Unavailable option" disabled:true { label "Unavailable option" }
```

</td></tr></table>

#### Disabled

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<AnimatedCheckbox title="Unavailable option" disabled />
```

</td><td>

```cronus
(see animated-checkbox.cronus)
```

</td></tr></table>

### RadioGroup (`/radio-group`)

#### Options

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const options = [
  { value: "starter", label: "Starter", hint: "For side projects" },
  { value: "pro", label: "Pro", hint: "For growing teams" },
  { value: "enterprise", label: "Enterprise", hint: "For large organizations" },
];

function RadioGroupDemo() {
  const [plan, setPlan] = useState("pro");

  return (
    <RadioGroup value={plan} onValueChange={setPlan} className="flex flex-col gap-3">
      {options.map((option) => (
        <div key={option.value} className="flex items-center gap-3">
          <RadioGroupItem value={option.value} id={`plan-${option.value}`} />
          <Label htmlFor={`plan-${option.value}`} className="flex flex-col gap-0.5">
            <span>{option.label}</span>
            <span className="text-xs font-normal text-fg-tertiary">{option.hint}</span>
          </Label>
        </div>
      ))}
    </RadioGroup>
  );
}
```

</td><td>

```cronus
component Plan layout:stack style:radio-group value:"Pro" {
  label "Plan"
  item "Starter" hint:"For side projects"
  item "Pro" hint:"For growing teams"
  item "Enterprise" hint:"For large organizations"
}
```

</td></tr></table>

### Switch (`/switch`)

#### Default

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function SwitchDemo() {
  const [enabled, setEnabled] = useState(true);

  return (
    <div className="flex items-center justify-between gap-3 max-w-xs">
      <Label htmlFor="notifications">Push notifications</Label>
      <Switch id="notifications" checked={enabled} onCheckedChange={setEnabled} />
    </div>
  );
}
```

</td><td>

```cronus
component Notifications layout:inline style:switch checked:true { label "Push notifications" text "Push notifications" }
```

</td></tr></table>

### Select (`/select`)

#### Grouped

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function SelectDemo() {
  const [region, setRegion] = useState<string | undefined>();

  return (
    <Field className="max-w-xs">
      <Label htmlFor="region">Deploy region</Label>
      <Select value={region ?? ""} onValueChange={setRegion}>
        <SelectTrigger id="region">
          <SelectValue placeholder="Choose a region" />
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            <SelectLabel>Americas</SelectLabel>
            <SelectItem value="us-east-1">US East (N. Virginia)</SelectItem>
            <SelectItem value="us-west-2">US West (Oregon)</SelectItem>
            <SelectItem value="sa-east-1">São Paulo</SelectItem>
          </SelectGroup>
          <SelectSeparator />
          <SelectGroup>
            <SelectLabel>Europe</SelectLabel>
            <SelectItem value="eu-west-1">Ireland</SelectItem>
            <SelectItem value="eu-central-1">Frankfurt</SelectItem>
          </SelectGroup>
        </SelectContent>
      </Select>
      <FieldDescription>
        {region ? `Selected: ${region}` : "No region selected yet."}
      </FieldDescription>
    </Field>
  );
}
```

</td><td>

```cronus
component Region layout:stack style:select {
  label "Choose a region"
  item "US East (N. Virginia)" group:"Americas"
  item "US West (Oregon)" group:"Americas"
  item "São Paulo" group:"Americas"
  item "Ireland" group:"Europe"
  item "Frankfurt" group:"Europe"
}
```

</td></tr></table>

### Combobox (`/combobox`)

#### Single select

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const frameworkOptions = [
  { value: "next", label: "Next.js" },
  { value: "remix", label: "Remix" },
  { value: "astro", label: "Astro" },
  { value: "nuxt", label: "Nuxt" },
  { value: "sveltekit", label: "SvelteKit" },
  { value: "solidstart", label: "SolidStart", disabled: true },
];

function ComboboxDemo() {
  const [framework, setFramework] = useState<string>();

  return (
    <Field className="max-w-xs">
      <FieldLabel id="framework-label">Framework</FieldLabel>
      <Combobox
        options={frameworkOptions}
        value={framework}
        onValueChange={setFramework}
        placeholder="Select a framework…"
        searchPlaceholder="Search frameworks…"
        aria-labelledby="framework-label"
      />
      <FieldDescription>
        {framework
          ? `Selected: ${frameworkOptions.find((o) => o.value === framework)?.label}`
          : "Search and pick a single framework."}
      </FieldDescription>
    </Field>
  );
}
```

</td><td>

```cronus
component Framework layout:stack style:combobox placeholder:"Select a framework…" search-placeholder:"Search frameworks…" aria-label:"Framework" {
  item "Next.js"
  item "Remix"
  item "Astro"
  item "Nuxt"
  item "SvelteKit"
  item "SolidStart" disabled:true
}
```

</td></tr></table>

### MultiSelect (`/multi-select`)

#### Default

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const skillOptions = [
  { value: "typescript", label: "TypeScript" },
  { value: "react", label: "React" },
  { value: "node", label: "Node.js" },
  { value: "postgres", label: "PostgreSQL" },
  { value: "tailwind", label: "Tailwind CSS" },
  { value: "graphql", label: "GraphQL" },
  { value: "rust", label: "Rust", disabled: true },
];

function MultiSelectDemo() {
  const [skills, setSkills] = useState<string[]>(["typescript", "react"]);

  return (
    <Field className="max-w-sm">
      <FieldLabel id="skills-label">Skills</FieldLabel>
      <MultiSelect
        options={skillOptions}
        value={skills}
        onValueChange={setSkills}
        placeholder="Select skills…"
        aria-labelledby="skills-label"
      />
      <FieldDescription>
        {skills.length > 0 ? `${skills.length} selected.` : "Pick one or more skills."}
      </FieldDescription>
    </Field>
  );
}
```

</td><td>

```cronus
component Skills layout:stack style:multi-select value:"TypeScript,React" placeholder:"Select skills…" aria-label:"Skills" {
  item "TypeScript"
  item "React"
  item "Node.js"
  item "PostgreSQL"
  item "Tailwind CSS"
  item "GraphQL"
  item "Rust" disabled:true
}
```

</td></tr></table>

#### Max display

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function MultiSelectMaxDisplayDemo() {
  const [skills, setSkills] = useState<string[]>([
    "typescript",
    "react",
    "node",
    "postgres",
  ]);

  return (
    <MultiSelect
      options={skillOptions}
      value={skills}
      onValueChange={setSkills}
      maxDisplay={2}
      placeholder="Select skills…"
      aria-label="Skills"
      className="max-w-sm"
    />
  );
}
```

</td><td>

```cronus
component SkillsCapped layout:stack style:multi-select value:"TypeScript,React,Node.js,PostgreSQL" max-display:2 placeholder:"Select skills…" aria-label:"Skills" {
  item "TypeScript"
  item "React"
  item "Node.js"
  item "PostgreSQL"
  item "Tailwind CSS"
}
```

</td></tr></table>

### TagsInput (`/tags-input`)

#### Default

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function TagsInputDemo() {
  const [tags, setTags] = useState<string[]>(["design", "react"]);
  const labelId = useId();

  return (
    <div className="flex w-full max-w-sm flex-col gap-2">
      <FieldLabel id={labelId}>Topics</FieldLabel>
      <TagsInput
        value={tags}
        onValueChange={setTags}
        aria-labelledby={labelId}
        max={6}
        placeholder="Add a topic…"
      />
      <FieldDescription>
        Press Enter or comma to add. Backspace removes the last tag.
      </FieldDescription>
    </div>
  );
}
```

</td><td>

```cronus
component Topics layout:stack style:tags-input placeholder:"Add a topic…" max:6 aria-label:"Topics" {
  text "design"
  text "react"
}
```

</td></tr></table>

### Slider (`/slider`)

#### Single thumb

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function SliderDemo() {
  const [volume, setVolume] = useState([40]);

  return (
    <div className="flex flex-col gap-4 max-w-xs">
      <div className="flex items-center justify-between text-sm">
        <Label>Volume</Label>
        <span className="font-mono text-fg">{volume[0]}%</span>
      </div>
      <Slider
        aria-label="Volume"
        value={volume}
        onValueChange={setVolume}
        min={0}
        max={100}
        step={1}
      />
    </div>
  );
}
```

</td><td>

```cronus
component Volume layout:stack style:slider value:40 aria-label:"Volume" { label "Volume" }
```

</td></tr></table>

#### Range

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function SliderRangeDemo() {
  const [price, setPrice] = useState([20, 80]);

  return (
    <div className="flex flex-col gap-4 max-w-xs">
      <div className="flex items-center justify-between text-sm">
        <Label>Price range</Label>
        <span className="font-mono text-fg">
          ${price[0]} – ${price[1]}
        </span>
      </div>
      <Slider
        aria-label="Price range"
        value={price}
        onValueChange={setPrice}
        min={0}
        max={100}
        step={5}
      />
    </div>
  );
}
```

</td><td>

```cronus
component Price layout:stack style:slider value:"20,80" aria-label:"Price range" { label "Price range" }
```

</td></tr></table>

### Field (`/field`)

#### Composition

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Field>
  <FieldLabel htmlFor="workspace">Workspace name</FieldLabel>
  <Input
    id="workspace"
    placeholder="acme-inc"
    aria-describedby="workspace-desc"
  />
  <FieldDescription id="workspace-desc">
    Used in your workspace URL. Letters, numbers, and dashes.
  </FieldDescription>
</Field>
```

</td><td>

```cronus
component WorkspaceField layout:stack style:field control:input placeholder:"acme-inc" description:"Used in your workspace URL. Letters, numbers, and dashes." { label "Workspace name" }
```

</td></tr></table>

#### With error

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function FieldErrorDemo() {
  const [username, setUsername] = useState("");
  const error = username.includes(" ") ? "Username can't contain spaces." : "";

  return (
    <Field>
      <FieldLabel htmlFor="username">Username</FieldLabel>
      <Input
        id="username"
        value={username}
        invalid={!!error}
        placeholder="ada"
        aria-describedby={error ? "username-err" : undefined}
        onChange={(event) => setUsername(event.target.value)}
      />
      <FieldError id="username-err">{error}</FieldError>
    </Field>
  );
}
```

</td><td>

```cronus
(see field.cronus)
```

</td></tr></table>

### Form (`/form`)

#### Validated form

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const profileSchema = z.object({
  email: z.string().email("Enter a valid email address."),
  bio: z.string().min(10, "Bio must be at least 10 characters."),
});

type ProfileValues = z.infer<typeof profileSchema>;

function FormDemo() {
  const [submitted, setSubmitted] = useState(false);

  const form = useForm<ProfileValues>({
    resolver: zodResolver(profileSchema),
    defaultValues: { email: "", bio: "" },
    mode: "onChange",
  });

  const onSubmit = (values: ProfileValues) => {
    setSubmitted(true);
  };

  return (
    <Form {...form}>
      <form
        onSubmit={form.handleSubmit(onSubmit)}
        className="flex max-w-md flex-col gap-5"
        noValidate
      >
        <FormField
          control={form.control}
          name="email"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Email</FormLabel>
              <FormControl>
                <Input type="email" placeholder="you@cronus.dev" {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="bio"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Bio</FormLabel>
              <FormControl>
                <Textarea rows={3} placeholder="Tell us a bit about yourself…" {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <div className="flex items-center gap-3">
          <Button type="submit">Save profile</Button>
          {submitted ? (
            <Badge variant="success">
              <Check aria-hidden="true" />
              Saved
            </Badge>
          ) : null}
        </div>
      </form>
    </Form>
  );
}
```

</td><td>

```cronus
component Profile layout:stack style:form {
  item "Email" type:email placeholder:"you@cronus.dev"
  item "Bio" textarea:true placeholder:"Tell us a bit about yourself…"
  action "Save profile"
}
```

</td></tr></table>

### InputOTP (`/input-otp`)

#### 6-digit

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function InputOTPDemo() {
  const [otp, setOtp] = useState("");

  return (
    <div className="flex flex-col gap-4">
      <InputOTP maxLength={6} value={otp} onChange={setOtp} aria-label="6-digit verification code">
        <InputOTPGroup>
          <InputOTPSlot index={0} />
          <InputOTPSlot index={1} />
          <InputOTPSlot index={2} />
        </InputOTPGroup>
        <InputOTPSeparator />
        <InputOTPGroup>
          <InputOTPSlot index={3} />
          <InputOTPSlot index={4} />
          <InputOTPSlot index={5} />
        </InputOTPGroup>
      </InputOTP>
      {otp.length === 6 ? (
        <Badge variant="success">
          <Check aria-hidden="true" />
          Code complete
        </Badge>
      ) : (
        <p className="text-xs text-fg-tertiary">Entered {otp.length} of 6 digits.</p>
      )}
    </div>
  );
}
```

</td><td>

```cronus
component Verification layout:inline style:input-otp aria-label:"6-digit verification code" { label "Verification code" }
```

</td></tr></table>

### FileDropzone (`/file-dropzone`)

#### Upload

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function FileDropzoneDemo() {
  const [files, setFiles] = useState<string[]>([]);

  return (
    <div className="flex flex-col gap-4">
      <FileDropzone
        multiple
        aria-label="Upload attachments"
        onFiles={(picked) => setFiles(picked.map((file) => file.name))}
      />
      {files.length > 0 ? (
        <ul className="flex flex-col gap-2">
          {files.map((name) => (
            <li
              key={name}
              className="flex items-center gap-2 rounded-lg border border-border-soft bg-surface-inset px-3 py-2 text-sm text-fg-secondary"
            >
              <FileText className="size-4 text-fg-tertiary" aria-hidden="true" />
              {name}
            </li>
          ))}
        </ul>
      ) : (
        <p className="text-xs text-fg-tertiary">No files selected yet.</p>
      )}
    </div>
  );
}
```

</td><td>

```cronus
component Attachments layout:stack style:file-dropzone multiple:true aria-label:"Upload attachments" { label "Upload attachments" }
```

</td></tr></table>

### NumberInput (`/number-input`)

#### Default

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function NumberInputDemo() {
  const [quantity, setQuantity] = useState<number | null>(1);

  return (
    <Field className="max-w-[12rem]">
      <FieldLabel htmlFor="qty">Quantity</FieldLabel>
      <NumberInput
        id="qty"
        value={quantity}
        onValueChange={setQuantity}
        min={0}
        max={10}
        aria-label="Quantity"
      />
      <FieldDescription>Between 0 and 10.</FieldDescription>
    </Field>
  );
}
```

</td><td>

```cronus
component Quantity layout:stack style:number-input value:1 min:0 max:10 aria-label:"Quantity" { label "Quantity" }
```

</td></tr></table>

#### Precision & formatting

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function NumberInputCurrencyDemo() {
  const [price, setPrice] = useState<number | null>(19.9);

  return (
    <NumberInput
      value={price}
      onValueChange={setPrice}
      min={0}
      step={0.1}
      precision={2}
      format={(value) => `$${value.toFixed(2)}`}
      aria-label="Price"
      className="max-w-[12rem]"
    />
  );
}
```

</td><td>

```cronus
component PriceNumber layout:stack style:number-input value:19.9 min:0 step:0.1 precision:2 prefix:"$" aria-label:"Price" { label "Price" }
```

</td></tr></table>

### Autocomplete (`/autocomplete`)

#### Default

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const frameworks: AutocompleteOption[] = [
  { value: "Next.js" },
  { value: "Remix" },
  { value: "Astro" },
  { value: "SvelteKit" },
  { value: "Nuxt" },
  { value: "SolidStart" },
];

function AutocompleteDemo() {
  const [value, setValue] = useState("");

  return (
    <Autocomplete
      options={frameworks}
      value={value}
      onValueChange={setValue}
      placeholder="Search a framework…"
      aria-label="Framework"
      className="max-w-xs"
    />
  );
}
```

</td><td>

```cronus
component FrameworkSearch layout:stack style:autocomplete placeholder:"Search a framework…" aria-label:"Framework" {
  item "Next.js"
  item "Remix"
  item "Astro"
  item "SvelteKit"
  item "Nuxt"
  item "SolidStart"
}
```

</td></tr></table>

#### Async suggestions

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function AutocompleteAsyncDemo() {
  const [value, setValue] = useState("");

  // Debounced; the component tracks its own loading state when onSearch is set.
  const search = async (query: string): Promise<AutocompleteOption[]> => {
    const res = await fetch(`/api/frameworks?q=${encodeURIComponent(query)}`);
    return res.json();
  };

  return (
    <Autocomplete
      onSearch={search}
      value={value}
      onValueChange={setValue}
      placeholder="Search (async)…"
      aria-label="Framework (async)"
      className="max-w-xs"
    />
  );
}
```

</td><td>

```cronus
(see autocomplete.cronus)
```

</td></tr></table>

### Stepper (`/stepper`)

#### Wizard

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const STEPPER_STEPS = [
  { title: "Account", description: "Your details" },
  { title: "Shipping", description: "Where to send it" },
  { title: "Payment", description: "Confirm & pay" },
];

function StepperDemo() {
  const [step, setStep] = useState(1);
  const lastStep = STEPPER_STEPS.length - 1;

  return (
    <div className="flex w-full max-w-xl flex-col gap-6">
      <Stepper value={step} onValueChange={setStep}>
        <StepperList>
          {STEPPER_STEPS.map((item, index) => (
            <StepperItem key={item.title} step={index}>
              <StepperTrigger>
                <StepperIndicator />
                <span className="flex flex-col">
                  <StepperTitle>{item.title}</StepperTitle>
                  <StepperDescription>{item.description}</StepperDescription>
                </span>
              </StepperTrigger>
              {index < lastStep ? <StepperSeparator /> : null}
            </StepperItem>
          ))}
        </StepperList>
      </Stepper>
      <div className="flex justify-between">
        <Button
          variant="outline"
          disabled={step === 0}
          onClick={() => setStep((value) => Math.max(0, value - 1))}
        >
          Back
        </Button>
        <Button
          disabled={step === lastStep}
          onClick={() => setStep((value) => Math.min(lastStep, value + 1))}
        >
          Next
        </Button>
      </div>
    </div>
  );
}
```

</td><td>

```cronus
component Checkout layout:stack style:stepper value:1 separators:true {
  item "Account" description:"Your details"
  item "Shipping" description:"Where to send it"
  item "Payment" description:"Confirm & pay"
}
```

</td></tr></table>

### RichTextEditor (`/rich-text-editor`)

#### Default

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function RichTextEditorDemo() {
  const [html, setHtml] = useState(
    "<h2>Release notes</h2><p>We shipped a <strong>themeable</strong> editor with a full toolbar — try <em>bold</em>, lists, and headings.</p><ul><li>Keyboard shortcuts</li><li>Undo &amp; redo</li></ul>",
  );

  return (
    <RichTextEditor
      value={html}
      onChange={setHtml}
      placeholder="Write something…"
      aria-label="Post body"
      className="max-w-xl"
    />
  );
}
```

</td><td>

```cronus
component Notes layout:stack style:rich-text-editor aria-label:"Post body" placeholder:"Write something…" {
  title "Release notes"
  text "We shipped a themeable editor with a full toolbar — try bold, lists, and headings."
  item "Keyboard shortcuts"
  item "Undo & redo"
}
```

</td></tr></table>

### Rating (`/rating`)

#### Interactive

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function RatingDemo() {
  const [value, setValue] = useState(3);

  return (
    <div className="flex flex-col items-center gap-3">
      <Rating value={value} onValueChange={setValue} aria-label="Rate your experience" />
      <p className="text-sm text-fg-secondary tabular-nums">
        {value} {value === 1 ? "star" : "stars"}
      </p>
    </div>
  );
}
```

</td><td>

```cronus
component Experience layout:inline style:rating value:3 interactive:true aria-label:"Rate your experience" { label "Rate your experience" }
```

</td></tr></table>

#### Read-only with a count

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex items-center gap-2">
  <Rating value={4.5} readOnly allowHalf size="sm" aria-label="Rated 4.5 out of 5" />
  <span className="text-sm text-fg-secondary">4.5 · 1,284 reviews</span>
</div>
```

</td><td>

```cronus
component Score layout:inline style:rating+sm value:4.5 readOnly:true aria-label:"Rated 4.5 out of 5" { label "Rated 4.5 out of 5" }
```

</td></tr></table>

### ColorPicker (`/color-picker`)

#### With swatches

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const brandSwatches = [
  "oklch(0.62 0.21 256)",
  "oklch(0.65 0.24 24)",
  "oklch(0.72 0.19 145)",
  "oklch(0.8 0.16 86)",
  "oklch(0.62 0.25 304)",
];

function ColorPickerDemo() {
  const [color, setColor] = useState("oklch(0.62 0.21 256)");

  return (
    <div className="flex w-full max-w-xs flex-col gap-3">
      <ColorPicker
        value={color}
        onValueChange={setColor}
        swatches={brandSwatches}
        aria-label="Brand color"
      />
      <div className="flex items-center gap-2 text-sm text-fg-secondary">
        <span
          aria-hidden="true"
          className="size-4 rounded-full border border-border"
          style={{ background: color }}
        />
        <span className="font-mono text-xs tabular-nums">{color}</span>
      </div>
    </div>
  );
}
```

</td><td>

```cronus
component Brand layout:stack style:color-picker value:"oklch(0.62 0.21 256)" aria-label:"Brand color" {
  label "Brand color"
  item "oklch(0.62 0.21 256)"
  item "oklch(0.65 0.24 24)"
  item "oklch(0.72 0.19 145)"
  item "oklch(0.8 0.16 86)"
  item "oklch(0.62 0.25 304)"
}
```

</td></tr></table>

### CurrencyInput (`/currency-input`)

#### Multi-currency

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function CurrencyInputDemo() {
  const [cents, setCents] = useState<number | null>(129900);
  const [meta, setMeta] = useState<CurrencyInputMeta | null>(null);

  return (
    <div className="flex w-full max-w-xs flex-col gap-2">
      <FieldLabel htmlFor="amount">Amount</FieldLabel>
      <CurrencyInput
        id="amount"
        value={cents}
        onValueChange={(next, info) => {
          setCents(next);
          setMeta(info);
        }}
        aria-label="Amount"
      />
      <FieldDescription>
        {cents == null
          ? "Digits fill in from the right — try typing 5 0 0 0 0."
          : `${meta?.currency ?? "BRL"} · ${(cents / 100).toLocaleString("en-US", {
              minimumFractionDigits: 2,
            })} in major units`}
      </FieldDescription>
    </div>
  );
}
```

</td><td>

```cronus
component Amount layout:stack style:currency-input value:129900 aria-label:"Amount" { label "Amount" }
```

</td></tr></table>

#### Single currency

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<CurrencyInput
  currencies={[{ code: "BRL", symbol: "R$", locale: "pt-BR" }]}
  defaultValue={12900}
  aria-label="Price"
/>
```

</td><td>

```cronus
component PriceBRL layout:stack style:currency-input currencies:"BRL" value:12900 aria-label:"Price" { label "Price" }
```

</td></tr></table>

#### Hard ceiling

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function CurrencyLimitDemo() {
  // R$ 5.000,00 hard ceiling, expressed in minor units (cents).
  const MAX_CENTS = 500000;
  const [cents, setCents] = useState<number | null>(320000);

  return (
    <div className="flex w-full max-w-xs flex-col gap-2">
      <div className="flex items-center justify-between">
        <FieldLabel htmlFor="budget">Monthly budget</FieldLabel>
        <span className="text-xs text-fg-tertiary tabular-nums">Máx R$ 5.000,00</span>
      </div>
      <CurrencyInput
        id="budget"
        currencies={[{ code: "BRL", symbol: "R$", locale: "pt-BR" }]}
        value={cents}
        onValueChange={setCents}
        max={MAX_CENTS}
        aria-label="Monthly budget"
      />
      <FieldDescription>A hard live ceiling — you can’t type past the limit.</FieldDescription>
    </div>
  );
}
```

</td><td>

```cronus
component Budget layout:stack style:currency-input currencies:"BRL" value:320000 max:500000 aria-label:"Monthly budget" { label "Monthly budget" }
```

</td></tr></table>

### PhoneInput (`/phone-input`)

#### Default

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function PhoneInputDemo() {
  const [phone, setPhone] = useState("");

  return (
    <div className="flex w-full max-w-xs flex-col gap-2">
      <FieldLabel htmlFor="phone">Phone number</FieldLabel>
      <PhoneInput id="phone" value={phone} onValueChange={setPhone} />
      <FieldDescription>
        {phone ? (
          <span className="font-mono text-xs tabular-nums text-fg-secondary">{phone}</span>
        ) : (
          "Pick a country and type — we compose the E.164 string."
        )}
      </FieldDescription>
    </div>
  );
}
```

</td><td>

```cronus
component Phone layout:stack style:phone-input default-country:BR aria-label:"Phone number" { label "Phone number" }
```

</td></tr></table>

#### Default country & value

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<PhoneInput defaultValue="+12025550142" aria-label="Phone number" />
```

</td><td>

```cronus
component Seeded layout:stack style:phone-input value:"+12025550142" aria-label:"Phone number" { label "Phone number" }
```

</td></tr></table>

#### Invalid

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Field>
  <FieldLabel htmlFor="phone">Phone number</FieldLabel>
  <PhoneInput id="phone" invalid defaultValue="+551198" aria-describedby="phone-err" />
  <FieldError id="phone-err">Enter a complete phone number.</FieldError>
</Field>
```

</td><td>

```cronus
component PhoneInvalid layout:stack style:phone-input value:"+551198" invalid:true aria-label:"Phone number" { label "Phone number" }
```

</td></tr></table>

### CreditCardInput (`/credit-card-input`)

#### Default

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function CreditCardDemo() {
  const [card, setCard] = useState<CreditCardValue | null>(null);

  return (
    <div className="flex w-full max-w-sm flex-col gap-3">
      <CreditCardInput label="Card details" onValueChange={setCard} />
      <div className="flex min-h-6 items-center text-xs text-fg-tertiary">
        {card?.valid ? (
          <Badge variant="success">
            <Check aria-hidden="true" />
            Valid card
          </Badge>
        ) : (
          <span className="capitalize">
            {card && card.brand !== "unknown"
              ? `${card.brand} detected`
              : "Try 4242 4242 4242 4242 · 12/34 · 123"}
          </span>
        )}
      </div>
    </div>
  );
}
```

</td><td>

```cronus
component Card layout:stack style:credit-card-input { label "Card details" }
```

</td></tr></table>

#### Brand detection

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<CreditCardInput label="Card details" defaultNumber="4242 4242 4242 4242" />
```

</td><td>

```cronus
component Visa layout:stack style:credit-card-input number:"4242 4242 4242 4242" { label "Card details" }
```

</td></tr></table>

#### With error

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<CreditCardInput
  label="Card details"
  defaultNumber="4000 0000 0000 0002"
  error="Your card was declined. Try another card."
/>
```

</td><td>

```cronus
component Declined layout:stack style:credit-card-input number:"4000 0000 0000 0002" error:"Your card was declined. Try another card." { label "Card details" }
```

</td></tr></table>

### FloatingLabelInput (`/floating-label-input`)

#### Default

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<FloatingLabelInput
  label="Email address"
  type="email"
  helperText="We'll never share your email."
/>
```

</td><td>

```cronus
component FloatEmail layout:stack style:floating-label-input type:email helper:"We'll never share your email." { label "Email address" }
```

</td></tr></table>

#### With adornments

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex flex-col gap-4">
  <FloatingLabelInput
    label="Email address"
    type="email"
    defaultValue="ada@cronus.dev"
    startAdornment={<Mail />}
  />
  <FloatingLabelInput label="Password" type="password" startAdornment={<Lock />} />
</div>
```

</td><td>

```cronus
component FloatEmailIcon layout:stack style:floating-label-input type:email value:"ada@cronus.dev" icon:mail { label "Email address" }
component FloatPassword layout:stack style:floating-label-input type:password icon:lock { label "Password" }
```

</td></tr></table>

#### Invalid

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<FloatingLabelInput
  label="Password"
  type="password"
  defaultValue="123"
  invalid
  startAdornment={<Lock />}
  helperText="Use at least 8 characters."
/>
```

</td><td>

```cronus
component FloatInvalid layout:stack style:floating-label-input type:password value:"123" invalid:true icon:lock helper:"Use at least 8 characters." { label "Password" }
```

</td></tr></table>

### SignaturePad (`/signature-pad`)

#### Capture a signature

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function SignaturePadDemo() {
  const [signature, setSignature] = useState<string | null>(null);

  return (
    <div className="flex w-full max-w-sm flex-col gap-3">
      <SignaturePad onChange={setSignature} aria-label="Contract signature" />
      <div className="flex h-9 items-center justify-between gap-3 text-xs text-fg-tertiary">
        <span>{signature ? "Signature captured" : "Awaiting signature"}</span>
        {signature ? (
          <img
            src={signature}
            alt="Captured signature preview"
            className="h-full rounded-md border border-border bg-surface-base"
          />
        ) : null}
      </div>
    </div>
  );
}
```

</td><td>

```cronus
component ContractSignature layout:stack style:signature-pad aria-label:"Contract signature" { label "Contract signature" }
```

</td></tr></table>

#### Disabled

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<SignaturePad disabled aria-label="Signature" />
```

</td><td>

```cronus
component FrozenSignature layout:stack style:signature-pad disabled:true aria-label:"Signature" { label "Signature" }
```

</td></tr></table>

### Chip (`/chip`)

#### Filter chips

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const TOPICS = ["Design", "React", "TypeScript", "Motion", "A11y"];

function ChipFilterDemo() {
  const [selected, setSelected] = useState<string[]>(["Design"]);

  const toggle = (topic: string) =>
    setSelected((current) =>
      current.includes(topic) ? current.filter((item) => item !== topic) : [...current, topic],
    );

  return (
    <ChipGroup aria-label="Filter articles by topic">
      {TOPICS.map((topic) => (
        <Chip key={topic} selected={selected.includes(topic)} onClick={() => toggle(topic)}>
          {topic}
        </Chip>
      ))}
    </ChipGroup>
  );
}
```

</td><td>

```cronus
component TopicFilters layout:inline style:chip aria-label:"Filter articles by topic" selected:false {
  item "Design" selected:true
  item "React"
  item "TypeScript"
  item "Motion"
  item "A11y"
}
```

</td></tr></table>

#### Variants, colors & sizes

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<ChipGroup aria-label="Chip variants">
  <Chip variant="solid" color="primary">Solid</Chip>
  <Chip variant="solid" color="error">Error</Chip>
  <Chip variant="soft" color="success">Soft</Chip>
  <Chip variant="soft" color="warning">Warning</Chip>
  <Chip variant="outline" color="info">Outline</Chip>
  <Chip variant="soft" size="sm">Small</Chip>
  <Chip variant="outline" size="lg">Large</Chip>
</ChipGroup>
```

</td><td>

```cronus
component Variants layout:inline style:chip aria-label:"Chip variants" {
  item "Solid" variant:solid color:primary
  item "Error" variant:solid color:error
  item "Soft" variant:soft color:success
  item "Warning" variant:soft color:warning
  item "Outline" variant:outline color:info
  item "Small" variant:soft size:sm
  item "Large" variant:outline size:lg
}
```

</td></tr></table>

#### Dismissible

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const INITIAL_TAGS = ["Aurora", "Nebula", "Pulsar", "Quasar"];

function ChipDismissibleDemo() {
  const [tags, setTags] = useState<string[]>(INITIAL_TAGS);

  if (tags.length === 0) {
    return (
      <Button variant="outline" size="sm" onClick={() => setTags(INITIAL_TAGS)}>
        Reset tags
      </Button>
    );
  }

  return (
    <ChipGroup aria-label="Selected tags">
      {tags.map((tag) => (
        <Chip
          key={tag}
          variant="soft"
          color="primary"
          onRemove={() => setTags((current) => current.filter((item) => item !== tag))}
        >
          {tag}
        </Chip>
      ))}
    </ChipGroup>
  );
}
```

</td><td>

```cronus
component Tags layout:inline style:chip+soft+primary aria-label:"Selected tags" removable:true {
  item "Aurora"
  item "Nebula"
  item "Pulsar"
  item "Quasar"
}
```

</td></tr></table>

## Display

### Avatar (`/avatar`)

#### Image & fallback

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex items-center gap-4">
  <Avatar>
    <AvatarImage src="https://github.com/shadcn.png" alt="@shadcn" />
    <AvatarFallback>CN</AvatarFallback>
  </Avatar>
  <Avatar>
    <AvatarFallback>AL</AvatarFallback>
  </Avatar>
  <Avatar>
    <AvatarFallback>CU</AvatarFallback>
  </Avatar>
</div>
```

</td><td>

```cronus
component AvatarShadcn layout:inline style:avatar src:"https://github.com/shadcn.png" alt:"@shadcn" { label "CN" }
component AvatarAda layout:inline style:avatar { label "AL" }
component AvatarCronus layout:inline style:avatar { label "CU" }
```

</td></tr></table>

#### Group

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex -space-x-3">
  {["CN", "AL", "JL", "MK"].map((initials) => (
    <Avatar key={initials} className="ring-2 ring-surface-raised">
      <AvatarFallback>{initials}</AvatarFallback>
    </Avatar>
  ))}
  <Avatar className="ring-2 ring-surface-raised">
    <AvatarFallback className="text-xs">+5</AvatarFallback>
  </Avatar>
</div>
```

</td><td>

```cronus
component AvatarStack layout:inline style:avatar ring:true {
  item "CN"
  item "AL"
  item "JL"
  item "MK"
  item "+5" size:xs
}
```

</td></tr></table>

### AvatarGroup (`/avatar-group`)

#### With overflow

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<AvatarGroup
  max={4}
  aria-label="Project collaborators"
  avatars={[
    { src: "https://github.com/shadcn.png", alt: "@shadcn", fallback: "CN" },
    { fallback: "AL" },
    { fallback: "JL" },
    { fallback: "MK" },
    { fallback: "RW" },
    { fallback: "TP" },
  ]}
/>
```

</td><td>

```cronus
component GroupCollaborators layout:inline style:avatar-group max:4 aria-label:"Project collaborators" {
  item "CN" src:"https://github.com/shadcn.png" alt:"@shadcn"
  item "AL"
  item "JL"
  item "MK"
  item "RW"
  item "TP"
}
```

</td></tr></table>

#### Sizes

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex flex-col gap-4">
  <AvatarGroup size="sm" max={3} aria-label="Reviewers" avatars={reviewers} />
  <AvatarGroup size="md" max={3} aria-label="Reviewers" avatars={reviewers} />
  <AvatarGroup size="lg" max={3} aria-label="Reviewers" avatars={reviewers} />
</div>
```

</td><td>

```cronus
component GroupReviewersSm layout:inline style:avatar-group+sm max:3 aria-label:"Reviewers" {
  item "CN" src:"https://github.com/shadcn.png" alt:"@shadcn"
  item "AL"
  item "JL"
  item "MK"
  item "RW"
}
component GroupReviewersMd layout:inline style:avatar-group+md max:3 aria-label:"Reviewers" {
  item "CN" src:"https://github.com/shadcn.png" alt:"@shadcn"
  item "AL"
  item "JL"
  item "MK"
  item "RW"
}
component GroupReviewersLg layout:inline style:avatar-group+lg max:3 aria-label:"Reviewers" {
  item "CN" src:"https://github.com/shadcn.png" alt:"@shadcn"
  item "AL"
  item "JL"
  item "MK"
  item "RW"
}
```

</td></tr></table>

### Badge (`/badge`)

#### Variants

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex flex-wrap gap-2">
  <Badge variant="default">Default</Badge>
  <Badge variant="primary">Primary</Badge>
  <Badge variant="secondary">Secondary</Badge>
  <Badge variant="outline">Outline</Badge>
  <Badge variant="success">Success</Badge>
  <Badge variant="warning">Warning</Badge>
  <Badge variant="error">Error</Badge>
  <Badge variant="info">Info</Badge>
</div>
```

</td><td>

```cronus
component BadgeDefault layout:inline style:badge+default { label "Default" }
component BadgePrimary layout:inline style:badge+primary { label "Primary" }
component BadgeSecondary layout:inline style:badge+secondary { label "Secondary" }
component BadgeOutline layout:inline style:badge+outline { label "Outline" }
component BadgeSuccess layout:inline style:badge+success { label "Success" }
component BadgeWarning layout:inline style:badge+warning { label "Warning" }
component BadgeError layout:inline style:badge+error { label "Error" }
component BadgeInfo layout:inline style:badge+info { label "Info" }
```

</td></tr></table>

#### With icon

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Badge variant="success">
  <Check aria-hidden="true" />
  Verified
</Badge>
```

</td><td>

```cronus
component BadgeVerified layout:inline style:badge+success icon:check { label "Verified" }
```

</td></tr></table>

### Card (`/card`)

#### Anatomy

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Card className="max-w-md">
  <CardHeader>
    <CardTitle>Pro plan</CardTitle>
    <CardDescription>Everything you need to ship a polished product.</CardDescription>
    <CardAction>
      <Badge variant="primary">Popular</Badge>
    </CardAction>
  </CardHeader>
  <CardContent className="flex items-baseline gap-1">
    <span className="font-display text-3xl font-semibold text-fg">$24</span>
    <span className="text-sm text-fg-tertiary">/ month</span>
  </CardContent>
  <CardFooter>
    <Button variant="primary" className="w-full">
      Upgrade now
    </Button>
  </CardFooter>
</Card>
```

</td><td>

```cronus
component CardProPlan layout:stack style:card max-width:md {
  title "Pro plan"
  text "Everything you need to ship a polished product."
  badge "Popular" variant:primary
  value "$24" unit:"/ month"
  action "Upgrade now" variant:primary width:full
}
```

</td></tr></table>

### GoalCard (`/goal-card`)

#### Progress

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<GoalCard
  id="mvp"
  title="Launch MVP by end of quarter"
  progress={75}
  status="in_progress"
  dueDate="2025-12-08"
  steps={[
    { id: "1", title: "Design", completed: true },
    { id: "2", title: "Develop", completed: true },
    { id: "3", title: "Ship", completed: false },
    { id: "4", title: "Announce", completed: false },
  ]}
/>
```

</td><td>

```cronus
component Mvp layout:stack style:goal-card+in_progress progress:75 due:"2025-12-08" {
  title "Launch MVP by end of quarter"
  item "Design" completed:true
  item "Develop" completed:true
  item "Ship"
  item "Announce"
}
```

</td></tr></table>

#### Design

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex w-full max-w-lg flex-col gap-3">
  <GoalCard id="a" title="Kickoff research" progress={0} status="not_started" />
  <GoalCard id="b" title="Launch MVP" progress={75} status="in_progress" />
  <GoalCard id="c" title="Complete onboarding" progress={100} status="completed" />
  <GoalCard id="d" title="Fix performance" progress={30} status="at_risk" />
</div>
```

</td><td>

```cronus
(see goal-card.cronus)
```

</td></tr></table>

#### Statuses

```cronus
component Kickoff layout:stack style:goal-card+not_started progress:0 { title "Kickoff research" }
component GoalLaunch layout:stack style:goal-card+in_progress progress:75 { title "Launch MVP" }
component Onboarding layout:stack style:goal-card+completed progress:100 { title "Complete onboarding" }
component Performance layout:stack style:goal-card+at_risk progress:30 { title "Fix performance" }
```

### Table (`/table`)

#### Basic

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Table>
  <TableCaption>A list of your recent invoices.</TableCaption>
  <TableHeader>
    <TableRow>
      <TableHead>Invoice</TableHead>
      <TableHead>Status</TableHead>
      <TableHead>Method</TableHead>
      <TableHead className="text-right">Amount</TableHead>
    </TableRow>
  </TableHeader>
  <TableBody>
    {invoices.map((invoice) => (
      <TableRow key={invoice.invoice}>
        <TableCell className="font-medium">{invoice.invoice}</TableCell>
        <TableCell>{invoice.status}</TableCell>
        <TableCell>{invoice.method}</TableCell>
        <TableCell className="text-right font-mono tabular-nums">{invoice.amount}</TableCell>
      </TableRow>
    ))}
  </TableBody>
  <TableFooter>
    <TableRow>
      <TableCell colSpan={3}>Total</TableCell>
      <TableCell className="text-right font-mono tabular-nums">$1,200.00</TableCell>
    </TableRow>
  </TableFooter>
</Table>
```

</td><td>

```cronus
component TableInvoices layout:stack style:table caption:"A list of your recent invoices." {
  columns "Invoice" font:medium
  columns "Status"
  columns "Method"
  columns "Amount" align:end font:mono
  text "INV-001"
  text "Paid"
  text "Credit Card"
  text "$250.00"
  text "INV-002"
  text "Pending"
  text "PayPal"
  text "$150.00"
  text "INV-003"
  text "Unpaid"
  text "Bank Transfer"
  text "$350.00"
  text "INV-004"
  text "Paid"
  text "Credit Card"
  text "$450.00"
  value "Total" span:3
  value "$1,200.00"
}
```

</td></tr></table>

### DataTable (`/data-table`)

#### Basic

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
type Payment = {
  id: string;
  status: "pending" | "processing" | "success" | "failed";
  email: string;
  amount: number;
};

const columns: ColumnDef<Payment>[] = [
  {
    accessorKey: "status",
    header: "Status",
    cell: ({ row }) => <span className="capitalize">{row.getValue("status")}</span>,
  },
  {
    accessorKey: "email",
    header: "Email",
    cell: ({ row }) => <span className="lowercase">{row.getValue("email")}</span>,
  },
  {
    accessorKey: "amount",
    header: ({ column }) => (
      <Button
        variant="ghost"
        size="sm"
        className="-ml-2"
        onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
      >
        Amount
        <ArrowUpDown aria-hidden="true" />
      </Button>
    ),
    cell: ({ row }) => {
      const amount = Number.parseFloat(String(row.getValue("amount")));
      const formatted = new Intl.NumberFormat("en-US", {
        style: "currency",
        currency: "USD",
      }).format(amount);
      return <span className="font-mono tabular-nums">{formatted}</span>;
    },
  },
];

const data: Payment[] = [
  { id: "m5gr84i9", amount: 316, status: "success", email: "ken99@example.com" },
  { id: "3u1reuv4", amount: 242, status: "success", email: "abe45@example.com" },
  { id: "derv1ws0", amount: 837, status: "processing", email: "monserrat44@example.com" },
  { id: "5kma53ae", amount: 874, status: "success", email: "silas22@example.com" },
  { id: "bhqecj4p", amount: 721, status: "failed", email: "carmella@example.com" },
  { id: "p0r9twq2", amount: 459, status: "pending", email: "jason.lee@example.com" },
];

<DataTable columns={columns} data={data} />
```

</td><td>

```cronus
component DataTableBasic layout:stack style:data-table {
  columns "Status" transform:capitalize
  columns "Email" transform:lowercase
  columns "Amount" sort:button font:mono
  item "success" email:"ken99@example.com" amount:"$316.00"
  item "success" email:"abe45@example.com" amount:"$242.00"
  item "processing" email:"monserrat44@example.com" amount:"$837.00"
  item "success" email:"silas22@example.com" amount:"$874.00"
  item "failed" email:"carmella@example.com" amount:"$721.00"
  item "pending" email:"jason.lee@example.com" amount:"$459.00"
}
```

</td></tr></table>

#### Sortable

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
import { DataTable, DataTableColumnHeader } from "@cronus-ui/ui";

const columns: ColumnDef<Member>[] = [
  {
    accessorKey: "name",
    header: ({ column }) => <DataTableColumnHeader column={column} title="Name" />,
    cell: ({ row }) => <span className="font-medium text-fg">{row.getValue("name")}</span>,
  },
  {
    accessorKey: "role",
    header: ({ column }) => <DataTableColumnHeader column={column} title="Role" />,
    cell: ({ row }) => <Badge variant="outline">{row.getValue("role")}</Badge>,
  },
  {
    accessorKey: "seats",
    header: ({ column }) => <DataTableColumnHeader column={column} title="Seats" />,
    cell: ({ row }) => (
      <span className="font-mono tabular-nums text-fg-secondary">{row.getValue("seats")}</span>
    ),
  },
];

<DataTable columns={columns} data={members} />
```

</td><td>

```cronus
component DataTableSortable layout:stack style:data-table {
  columns "Name" sort:true font:medium
  columns "Email" sort:true color:secondary
  columns "Role" sort:true badge:outline
  columns "Status" sort:true badge:tone transform:capitalize tones:"active=success,invited=warning,suspended=error"
  columns "Seats" sort:true font:mono color:secondary
  item "Ada Lovelace" email:"ada@cronus.dev" role:Owner status:active seats:5
  item "Grace Hopper" email:"grace@cronus.dev" role:Admin status:active seats:3
  item "Alan Turing" email:"alan@cronus.dev" role:Member status:active seats:1
  item "Katherine Johnson" email:"katherine@cronus.dev" role:Member status:invited seats:1
  item "Margaret Hamilton" email:"margaret@cronus.dev" role:Admin status:active seats:2
  item "Dennis Ritchie" email:"dennis@cronus.dev" role:Viewer status:suspended seats:0
}
```

</td></tr></table>

#### Search & filter

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const statusOptions = [
  { label: "Active", value: "active", icon: Check },
  { label: "Invited", value: "invited", icon: CircleDashed },
  { label: "Suspended", value: "suspended", icon: CircleSlash },
];

const statusFilter: DataTableFacetedFilter = {
  columnId: "status",
  title: "Status",
  options: statusOptions,
};

<DataTable
  columns={columns}
  data={members}
  searchable
  searchPlaceholder="Search members…"
  facetedFilters={[statusFilter]}
/>
```

</td><td>

```cronus
component DataTableSearch layout:stack style:data-table searchable:true search-placeholder:"Search members…" filter:Status filter-icons:"check,circle-dashed,circle-slash" {
  columns "Name" sort:true font:medium
  columns "Email" sort:true color:secondary
  columns "Role" sort:true badge:outline
  columns "Status" sort:true badge:tone transform:capitalize tones:"active=success,invited=warning,suspended=error"
  columns "Seats" sort:true font:mono color:secondary
  item "Ada Lovelace" email:"ada@cronus.dev" role:Owner status:active seats:5
  item "Grace Hopper" email:"grace@cronus.dev" role:Admin status:active seats:3
  item "Alan Turing" email:"alan@cronus.dev" role:Member status:active seats:1
  item "Katherine Johnson" email:"katherine@cronus.dev" role:Member status:invited seats:1
  item "Margaret Hamilton" email:"margaret@cronus.dev" role:Admin status:active seats:2
  item "Dennis Ritchie" email:"dennis@cronus.dev" role:Viewer status:suspended seats:0
  item "Barbara Liskov" email:"barbara@cronus.dev" role:Member status:active seats:1
  item "Donald Knuth" email:"donald@cronus.dev" role:Member status:invited seats:1
  item "Edsger Dijkstra" email:"edsger@cronus.dev" role:Viewer status:active seats:1
  item "Linus Torvalds" email:"linus@cronus.dev" role:Admin status:active seats:4
  item "Tim Berners-Lee" email:"tim@cronus.dev" role:Member status:suspended seats:0
  item "Radia Perlman" email:"radia@cronus.dev" role:Member status:active seats:2
}
```

</td></tr></table>

#### Pagination

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<DataTable
  columns={columns}
  data={members}
  pagination
  initialPageSize={5}
  pageSizeOptions={[5, 10, 20]}
/>
```

</td><td>

```cronus
component DataTablePagination layout:stack style:data-table pagination:true page-size:5 page-sizes:"5,10,20" {
  columns "Name" sort:true font:medium
  columns "Email" sort:true color:secondary
  columns "Role" sort:true badge:outline
  columns "Status" sort:true badge:tone transform:capitalize tones:"active=success,invited=warning,suspended=error"
  columns "Seats" sort:true font:mono color:secondary
  item "Ada Lovelace" email:"ada@cronus.dev" role:Owner status:active seats:5
  item "Grace Hopper" email:"grace@cronus.dev" role:Admin status:active seats:3
  item "Alan Turing" email:"alan@cronus.dev" role:Member status:active seats:1
  item "Katherine Johnson" email:"katherine@cronus.dev" role:Member status:invited seats:1
  item "Margaret Hamilton" email:"margaret@cronus.dev" role:Admin status:active seats:2
  item "Dennis Ritchie" email:"dennis@cronus.dev" role:Viewer status:suspended seats:0
  item "Barbara Liskov" email:"barbara@cronus.dev" role:Member status:active seats:1
  item "Donald Knuth" email:"donald@cronus.dev" role:Member status:invited seats:1
  item "Edsger Dijkstra" email:"edsger@cronus.dev" role:Viewer status:active seats:1
  item "Linus Torvalds" email:"linus@cronus.dev" role:Admin status:active seats:4
  item "Tim Berners-Lee" email:"tim@cronus.dev" role:Member status:suspended seats:0
  item "Radia Perlman" email:"radia@cronus.dev" role:Member status:active seats:2
}
```

</td></tr></table>

#### Selection & bulk actions

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<DataTable
  columns={columns}
  data={members}
  enableRowSelection
  pagination
  initialPageSize={5}
  bulkActions={(rows) => (
    <>
      <Button variant="outline" size="sm">
        <Mail aria-hidden="true" />
        Email {rows.length}
      </Button>
      <Button variant="destructive" size="sm">
        Remove
      </Button>
    </>
  )}
/>
```

</td><td>

```cronus
component DataTableSelection layout:stack style:data-table selection:true pagination:true page-size:5 {
  columns "Name" sort:true font:medium
  columns "Email" sort:true color:secondary
  columns "Role" sort:true badge:outline
  columns "Status" sort:true badge:tone transform:capitalize tones:"active=success,invited=warning,suspended=error"
  columns "Seats" sort:true font:mono color:secondary
  item "Ada Lovelace" email:"ada@cronus.dev" role:Owner status:active seats:5
  item "Grace Hopper" email:"grace@cronus.dev" role:Admin status:active seats:3
  item "Alan Turing" email:"alan@cronus.dev" role:Member status:active seats:1
  item "Katherine Johnson" email:"katherine@cronus.dev" role:Member status:invited seats:1
  item "Margaret Hamilton" email:"margaret@cronus.dev" role:Admin status:active seats:2
  item "Dennis Ritchie" email:"dennis@cronus.dev" role:Viewer status:suspended seats:0
  item "Barbara Liskov" email:"barbara@cronus.dev" role:Member status:active seats:1
  item "Donald Knuth" email:"donald@cronus.dev" role:Member status:invited seats:1
  item "Edsger Dijkstra" email:"edsger@cronus.dev" role:Viewer status:active seats:1
  item "Linus Torvalds" email:"linus@cronus.dev" role:Admin status:active seats:4
  item "Tim Berners-Lee" email:"tim@cronus.dev" role:Member status:suspended seats:0
  item "Radia Perlman" email:"radia@cronus.dev" role:Member status:active seats:2
  action "Email" icon:mail variant:outline count:true
  action "Remove" variant:destructive
}
```

</td></tr></table>

#### Column visibility

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<DataTable
  columns={columns}
  data={members}
  searchable
  enableColumnVisibility
/>
```

</td><td>

```cronus
component DataTableColumns layout:stack style:data-table searchable:true column-visibility:true {
  columns "Name" sort:true font:medium
  columns "Email" sort:true color:secondary
  columns "Role" sort:true badge:outline
  columns "Status" sort:true badge:tone transform:capitalize tones:"active=success,invited=warning,suspended=error"
  columns "Seats" sort:true font:mono color:secondary
  item "Ada Lovelace" email:"ada@cronus.dev" role:Owner status:active seats:5
  item "Grace Hopper" email:"grace@cronus.dev" role:Admin status:active seats:3
  item "Alan Turing" email:"alan@cronus.dev" role:Member status:active seats:1
  item "Katherine Johnson" email:"katherine@cronus.dev" role:Member status:invited seats:1
  item "Margaret Hamilton" email:"margaret@cronus.dev" role:Admin status:active seats:2
  item "Dennis Ritchie" email:"dennis@cronus.dev" role:Viewer status:suspended seats:0
}
```

</td></tr></table>

#### Density

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<DataTable
  columns={columns}
  data={members}
  searchable
  enableDensityToggle
/>
```

</td><td>

```cronus
component DataTableDensity layout:stack style:data-table searchable:true density-toggle:true {
  columns "Name" sort:true font:medium
  columns "Email" sort:true color:secondary
  columns "Role" sort:true badge:outline
  columns "Status" sort:true badge:tone transform:capitalize tones:"active=success,invited=warning,suspended=error"
  columns "Seats" sort:true font:mono color:secondary
  item "Ada Lovelace" email:"ada@cronus.dev" role:Owner status:active seats:5
  item "Grace Hopper" email:"grace@cronus.dev" role:Admin status:active seats:3
  item "Alan Turing" email:"alan@cronus.dev" role:Member status:active seats:1
  item "Katherine Johnson" email:"katherine@cronus.dev" role:Member status:invited seats:1
  item "Margaret Hamilton" email:"margaret@cronus.dev" role:Admin status:active seats:2
  item "Dennis Ritchie" email:"dennis@cronus.dev" role:Viewer status:suspended seats:0
}
```

</td></tr></table>

#### Loading

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const [loading, setLoading] = useState(true);

<DataTable
  columns={columns}
  data={members}
  loading={loading}
  loadingRowCount={4}
/>
```

</td><td>

```cronus
component DataTableLoading layout:stack style:data-table loading:true loading-rows:4 {
  columns "Name" sort:true font:medium
  columns "Email" sort:true color:secondary
  columns "Role" sort:true badge:outline
  columns "Status" sort:true badge:tone transform:capitalize tones:"active=success,invited=warning,suspended=error"
  columns "Seats" sort:true font:mono color:secondary
  item "Ada Lovelace" email:"ada@cronus.dev" role:Owner status:active seats:5
  item "Grace Hopper" email:"grace@cronus.dev" role:Admin status:active seats:3
  item "Alan Turing" email:"alan@cronus.dev" role:Member status:active seats:1
  item "Katherine Johnson" email:"katherine@cronus.dev" role:Member status:invited seats:1
}
```

</td></tr></table>

#### Empty

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<DataTable
  columns={columns}
  data={[]}
  emptyState={
    <Empty>
      <EmptyIcon>
        <Inbox aria-hidden="true" />
      </EmptyIcon>
      <EmptyTitle>No members yet</EmptyTitle>
      <EmptyDescription>Invite teammates to see them listed here.</EmptyDescription>
    </Empty>
  }
/>
```

</td><td>

```cronus
component DataTableEmpty layout:stack style:data-table empty-icon:inbox empty-title:"No members yet" empty-description:"Invite teammates to see them listed here." {
  columns "Name" sort:true font:medium
  columns "Email" sort:true color:secondary
  columns "Role" sort:true badge:outline
  columns "Status" sort:true badge:tone transform:capitalize
  columns "Seats" sort:true font:mono color:secondary
}
```

</td></tr></table>

#### Error

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const [failed, setFailed] = useState(true);

<DataTable
  columns={columns}
  data={failed ? [] : members}
  error={failed ? "Couldn't load team members. Check your connection and try again." : undefined}
  onRetry={() => setFailed(false)}
/>
```

</td><td>

```cronus
component DataTableError layout:stack style:data-table error:"Couldn’t load team members. Check your connection and try again." retry:true {
  columns "Name" sort:true font:medium
  columns "Email" sort:true color:secondary
  columns "Role" sort:true badge:outline
  columns "Status" sort:true badge:tone transform:capitalize
  columns "Seats" sort:true font:mono color:secondary
}
```

</td></tr></table>

### Metric (`/metric`)

#### Stat tiles

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="grid gap-4 sm:grid-cols-3">
  <Metric>
    <MetricLabel>Revenue</MetricLabel>
    <MetricValue>$48,290</MetricValue>
    <MetricDelta trend="up">+12.5%</MetricDelta>
  </Metric>
  <Metric>
    <MetricLabel>Churn</MetricLabel>
    <MetricValue>2.1%</MetricValue>
    <MetricDelta trend="down">-0.4%</MetricDelta>
  </Metric>
  <Metric>
    <MetricLabel>Sessions</MetricLabel>
    <MetricValue>9,830</MetricValue>
    <MetricDelta trend="neutral">0.0%</MetricDelta>
  </Metric>
</div>
```

</td><td>

```cronus
component MetricRevenue layout:stack style:metric {
  label "Revenue"
  value "$48,290"
  trend "+12.5%" tone:up
}
component MetricChurn layout:stack style:metric {
  label "Churn"
  value "2.1%"
  trend "-0.4%" tone:down
}
component MetricSessions layout:stack style:metric {
  label "Sessions"
  value "9,830"
  trend "0.0%" tone:neutral
}
```

</td></tr></table>

### Sparkline (`/sparkline`)

#### Line, area & bar

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex items-center gap-8">
  <Sparkline data={[4, 6, 5, 8, 7, 11, 9, 13]} tone="primary" />
  <Sparkline data={[4, 6, 5, 8, 7, 11, 9, 13]} tone="success" area />
  <Sparkline data={[4, 6, 5, 8, 7, 11, 9, 13]} type="bar" tone="info" />
</div>
```

</td><td>

```cronus
component SparklineLine layout:inline style:sparkline+primary data:"4,6,5,8,7,11,9,13" {}
component SparklineArea layout:inline style:sparkline+success+area data:"4,6,5,8,7,11,9,13" {}
component SparklineBar layout:inline style:sparkline+info+bar data:"4,6,5,8,7,11,9,13" {}
```

</td></tr></table>

#### In stat cards

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="grid gap-4 sm:grid-cols-3">
  <Card>
    <CardContent className="flex flex-col gap-3 pt-6">
      <Metric>
        <MetricLabel>Revenue</MetricLabel>
        <MetricValue>$48,290</MetricValue>
        <MetricDelta trend="up">+12.5%</MetricDelta>
      </Metric>
      <Sparkline
        data={[18, 22, 19, 27, 24, 31, 29, 38]}
        tone="success"
        area
        className="w-full"
        height={36}
        aria-label="Revenue, trending up"
      />
    </CardContent>
  </Card>
  <Card>
    <CardContent className="flex flex-col gap-3 pt-6">
      <Metric>
        <MetricLabel>Active users</MetricLabel>
        <MetricValue>9,830</MetricValue>
        <MetricDelta trend="up">+4.1%</MetricDelta>
      </Metric>
      <Sparkline
        data={[40, 38, 42, 41, 45, 44, 48, 52]}
        tone="primary"
        className="w-full"
        height={36}
        aria-label="Active users, trending up"
      />
    </CardContent>
  </Card>
  <Card>
    <CardContent className="flex flex-col gap-3 pt-6">
      <Metric>
        <MetricLabel>Churn</MetricLabel>
        <MetricValue>2.1%</MetricValue>
        <MetricDelta trend="down">-0.4%</MetricDelta>
      </Metric>
      <Sparkline
        data={[9, 8, 8, 7, 6, 6, 5, 4]}
        type="bar"
        tone="error"
        className="w-full"
        height={36}
        aria-label="Churn, trending down"
      />
    </CardContent>
  </Card>
</div>
```

</td><td>

```cronus
component SparklineRevenue layout:stack style:sparkline+success+area data:"18,22,19,27,24,31,29,38" height:36 full:true aria-label:"Revenue, trending up" {
  label "Revenue"
  value "$48,290"
  trend "+12.5%" tone:up
}
component SparklineUsers layout:stack style:sparkline+primary data:"40,38,42,41,45,44,48,52" height:36 full:true aria-label:"Active users, trending up" {
  label "Active users"
  value "9,830"
  trend "+4.1%" tone:up
}
component SparklineChurn layout:stack style:sparkline+error+bar data:"9,8,8,7,6,6,5,4" height:36 full:true aria-label:"Churn, trending down" {
  label "Churn"
  value "2.1%"
  trend "-0.4%" tone:down
}
```

</td></tr></table>

### Masonry (`/masonry`)

#### Responsive cards

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const cards = [
  { id: "m-1", title: "Onboarding", body: "Passwordless email code." },
  { id: "m-2", title: "Checkout", body: "Card and boleto with upsells that convert." },
  { id: "m-3", title: "Payouts", body: "Settled on a fixed schedule." },
  { id: "m-4", title: "Analytics", body: "Live revenue and conversion metrics." },
  { id: "m-5", title: "Members", body: "Grant and revoke access automatically." },
  { id: "m-6", title: "Notifications", body: "Push and email on every sale." },
];

return (
  <Masonry columns={{ base: 1, sm: 2, lg: 3 }} gap="1rem">
    {cards.map((card) => (
      <Card key={card.id}>
        <CardHeader>
          <CardTitle>{card.title}</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-fg-secondary">{card.body}</p>
        </CardContent>
      </Card>
    ))}
  </Masonry>
);
```

</td><td>

```cronus
component MasonryCards layout:stack style:masonry columns:"1,2,3" {
  item "Onboarding" description:"Passwordless email code."
  item "Checkout" description:"Card and boleto with upsells that convert."
  item "Payouts" description:"Settled on a fixed schedule."
  item "Analytics" description:"Live revenue and conversion metrics."
  item "Members" description:"Grant and revoke access automatically."
  item "Notifications" description:"Push and email on every sale."
}
```

</td></tr></table>

### ComparisonSlider (`/comparison-slider`)

#### Before & after

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<ComparisonSlider
  aria-label="Before and after"
  className="h-64"
  before={
    <div className="flex size-full items-center justify-center bg-surface-inset">
      <span className="rounded-md bg-surface-overlay px-3 py-1 text-sm font-medium text-fg">
        Before
      </span>
    </div>
  }
  after={
    <div className="flex size-full items-center justify-center bg-primary">
      <span className="rounded-md bg-surface-base/80 px-3 py-1 text-sm font-medium text-fg">
        After
      </span>
    </div>
  }
/>
```

</td><td>

```cronus
component ComparisonBeforeAfter layout:stack style:comparison-slider aria-label:"Before and after" height:64 before:"Before" after:"After" {}
```

</td></tr></table>

### Heatmap (`/heatmap`)

#### Contributions

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
// 17 weeks of daily activity, laid out into week columns of 7.
const data = Array.from({ length: 119 }, (_, i) => {
  const date = new Date(2026, 2, 1 + i);
  return {
    date: date.toISOString().slice(0, 10),
    value: Math.round(Math.random() * 12),
  };
});

return <Heatmap data={data} aria-label="Contributions" />;
```

</td><td>

```cronus
component HeatmapContributions layout:stack style:heatmap aria-label:"Contributions" start:"2026-03-01" data:"1,6,6,10,1,3,7,7,9,7,9,9,9,7,3,7,1,10,5,10,8,8,3,3,12,11,10,12,6,1,4,7,4,5,11,8,7,0,9,4,5,5,7,6,1,6,9,7,6,11,3,5,11,6,6,0,3,2,1,8,11,1,7,6,6,3,11,8,5,6,11,12,7,7,8,4,10,6,6,11,5,4,3,0,8,7,5,6,6,11,3,6,6,10,10,7,11,10,4,11,1,8,9,6,5,12,12,10,6,1,3,6,1,9,5,5,0,0,7" {}
```

</td></tr></table>

### Kbd (`/kbd`)

#### Keys

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex items-center gap-3">
  <span className="inline-flex items-center gap-1">
    <Kbd>⌘</Kbd>
    <Kbd>K</Kbd>
  </span>
  <Kbd>Esc</Kbd>
  <Kbd>↵</Kbd>
</div>
```

</td><td>

```cronus
component KbdCommandK layout:inline style:kbd {
  item "⌘"
  item "K"
}
component KbdEscape layout:inline style:kbd { label "Esc" }
component KbdEnter layout:inline style:kbd { label "↵" }
```

</td></tr></table>

### Empty (`/empty`)

#### Empty state

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Empty>
  <EmptyIcon>
    <Inbox aria-hidden="true" />
  </EmptyIcon>
  <EmptyTitle>No messages yet</EmptyTitle>
  <EmptyDescription>
    When someone sends you a message, it will show up here. Start a conversation to get going.
  </EmptyDescription>
  <EmptyContent>
    <Button>New message</Button>
  </EmptyContent>
</Empty>
```

</td><td>

```cronus
component EmptyNoMessages layout:stack style:empty icon:inbox {
  title "No messages yet"
  text "When someone sends you a message, it will show up here. Start a conversation to get going."
  action "New message"
}
```

</td></tr></table>

### Separator (`/separator`)

#### Horizontal & vertical

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex flex-col gap-3">
  <span className="text-sm text-fg-secondary">Section A</span>
  <Separator />
  <span className="text-sm text-fg-secondary">Section B</span>
  <div className="flex h-10 items-center gap-3 text-sm text-fg-secondary">
    <span>Docs</span>
    <Separator orientation="vertical" />
    <span>API</span>
    <Separator orientation="vertical" />
    <span>Blog</span>
  </div>
</div>
```

</td><td>

```cronus
component SeparatorSections layout:stack style:separator {
  text "Section A"
  text "Section B"
}
component SeparatorLinks layout:inline style:separator+vertical {
  text "Docs"
  text "API"
  text "Blog"
}
```

</td></tr></table>

### Skeleton (`/skeleton`)

#### Loading card

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex items-center gap-3">
  <Skeleton className="size-12 rounded-full" />
  <div className="flex flex-1 flex-col gap-2">
    <Skeleton className="h-3 w-3/4 rounded-md" />
    <Skeleton className="h-3 w-1/2 rounded-md" />
  </div>
</div>
```

</td><td>

```cronus
component SkeletonLoadingCard layout:inline style:skeleton {
  item "Avatar" shape:circle size:12
  item "Title" height:3 width:3/4
  item "Subtitle" height:3 width:1/2
}
```

</td></tr></table>

### ScrollArea (`/scroll-area`)

#### Scrollable list

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const tags = Array.from({ length: 20 }, (_, index) => `v1.2.0-beta.${20 - index}`);

<ScrollArea className="h-48 w-full max-w-xs rounded-xl border border-border-soft bg-surface-inset">
  <div className="flex flex-col gap-1 p-4">
    {tags.map((tag) => (
      <div key={tag} className="rounded-md px-2 py-1.5 font-mono text-sm text-fg-secondary">
        {tag}
      </div>
    ))}
  </div>
  <ScrollBar orientation="vertical" />
</ScrollArea>
```

</td><td>

```cronus
component ScrollAreaTags layout:stack style:scroll-area height:48 max-width:xs framed:true font:mono {
  item "v1.2.0-beta.20"
  item "v1.2.0-beta.19"
  item "v1.2.0-beta.18"
  item "v1.2.0-beta.17"
  item "v1.2.0-beta.16"
  item "v1.2.0-beta.15"
  item "v1.2.0-beta.14"
  item "v1.2.0-beta.13"
  item "v1.2.0-beta.12"
  item "v1.2.0-beta.11"
  item "v1.2.0-beta.10"
  item "v1.2.0-beta.9"
  item "v1.2.0-beta.8"
  item "v1.2.0-beta.7"
  item "v1.2.0-beta.6"
  item "v1.2.0-beta.5"
  item "v1.2.0-beta.4"
  item "v1.2.0-beta.3"
  item "v1.2.0-beta.2"
  item "v1.2.0-beta.1"
}
```

</td></tr></table>

### CodeBlock (`/code-block`)

#### Default

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<CodeBlock
  language="bash"
  filename="terminal"
  code="bunx cronus-ui add button card dialog"
/>
```

</td><td>

```cronus
component CodeBlockTerminal layout:stack style:code-block language:bash filename:terminal {
  text "bunx cronus-ui add button card dialog"
}
```

</td></tr></table>

#### Line numbers

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<CodeBlock
  language="tsx"
  showLineNumbers
  code={`import { Button } from "@cronus-ui/ui";

export function Save() {
  return <Button>Save</Button>;
}`}
/>
```

</td><td>

```cronus
component CodeBlockLines layout:stack style:code-block language:tsx line-numbers:true {
  text "import { Button } from \"@cronus-ui/ui\";"
  text ""
  text "export function Save() {"
  text "  return <Button>Save</Button>;"
  text "}"
}
```

</td></tr></table>

### CodeTabs (`/code-tabs`)

#### Package manager installer

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<CodeTabs
  storageKey="pkg-manager"
  items={[
    { label: "bun", code: "bunx cronus-ui add code-tabs", language: "bash" },
    { label: "npm", code: "npx cronus-ui add code-tabs", language: "bash" },
    { label: "pnpm", code: "pnpm dlx cronus-ui add code-tabs", language: "bash" },
    { label: "yarn", code: "yarn dlx cronus-ui add code-tabs", language: "bash" },
  ]}
/>
```

</td><td>

```cronus
component CodeTabsInstaller layout:stack style:code-tabs {
  tab "bun" code:"bunx cronus-ui add code-tabs" language:bash
  tab "npm" code:"npx cronus-ui add code-tabs" language:bash
  tab "pnpm" code:"pnpm dlx cronus-ui add code-tabs" language:bash
  tab "yarn" code:"yarn dlx cronus-ui add code-tabs" language:bash
}
```

</td></tr></table>

#### Multi-language snippet

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<CodeTabs
  defaultLabel="TypeScript"
  items={[
    {
      label: "TypeScript",
      language: "ts",
      code: `export function formatBRL(value: number): string {
  return new Intl.NumberFormat("pt-BR", {
    style: "currency",
    currency: "BRL",
  }).format(value);
}`,
    },
    {
      label: "JavaScript",
      language: "js",
      code: `export function formatBRL(value) {
  return new Intl.NumberFormat("pt-BR", {
    style: "currency",
    currency: "BRL",
  }).format(value);
}`,
    },
  ]}
/>
```

</td><td>

```cronus
component CodeTabsLanguages layout:stack style:code-tabs default:"TypeScript" {
  tab "TypeScript" language:ts
  text "export function formatBRL(value: number): string {"
  text "  return new Intl.NumberFormat(\"pt-BR\", {"
  text "    style: \"currency\","
  text "    currency: \"BRL\","
  text "  }).format(value);"
  text "}"
  tab "JavaScript" language:js
  text "export function formatBRL(value) {"
  text "  return new Intl.NumberFormat(\"pt-BR\", {"
  text "    style: \"currency\","
  text "    currency: \"BRL\","
  text "  }).format(value);"
  text "}"
}
```

</td></tr></table>

### Collapsible (`/collapsible`)

#### Default

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function CollapsibleDemo() {
  const [open, setOpen] = useState(false);

  return (
    <Collapsible open={open} onOpenChange={setOpen} className="w-full max-w-sm space-y-2">
      <div className="flex items-center justify-between gap-4 rounded-lg border border-border px-4 py-2">
        <span className="text-sm font-medium text-fg">Deployment regions</span>
        <CollapsibleTrigger asChild>
          <Button variant="ghost" size="icon-sm" aria-label="Toggle regions">
            <ChevronsUpDown />
          </Button>
        </CollapsibleTrigger>
      </div>
      <CollapsibleContent className="space-y-2">
        {["us-east-1", "eu-west-1", "ap-southeast-2"].map((region) => (
          <div key={region} className="rounded-lg border border-border px-4 py-2 font-mono text-sm">
            {region}
          </div>
        ))}
      </CollapsibleContent>
    </Collapsible>
  );
}
```

</td><td>

```cronus
component CollapsibleRegions layout:stack style:collapsible open:false icon:chevrons-up-down aria-label:"Toggle regions" max-width:sm font:mono {
  label "Deployment regions"
  item "us-east-1"
  item "eu-west-1"
  item "ap-southeast-2"
}
```

</td></tr></table>

### AspectRatio (`/aspect-ratio`)

#### Default

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<AspectRatio ratio={16 / 9} className="overflow-hidden rounded-xl border border-border">
  <img
    src="/og.png"
    alt="Cover"
    className="h-full w-full object-cover"
  />
</AspectRatio>
```

</td><td>

```cronus
component RatioCover layout:stack style:aspect-ratio ratio:"16/9" src:"/og.png" alt:"Cover" framed:true { label "Cover" }
```

</td></tr></table>

#### Square

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<AspectRatio ratio={1} className="overflow-hidden rounded-xl border border-border">
  <img src="/thumb.png" alt="Thumbnail" className="h-full w-full object-cover" />
</AspectRatio>
```

</td><td>

```cronus
component RatioThumbnail layout:stack style:aspect-ratio ratio:1 src:"/thumb.png" alt:"Thumbnail" framed:true { label "Thumbnail" }
```

</td></tr></table>

### TreeView (`/tree-view`)

#### File tree

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const data: TreeNode[] = [
  {
    id: "src",
    label: "src",
    icon: <Folder aria-hidden="true" />,
    children: [
      {
        id: "components",
        label: "components",
        icon: <Folder aria-hidden="true" />,
        children: [
          { id: "button.tsx", label: "button.tsx", icon: <FileText aria-hidden="true" /> },
          { id: "card.tsx", label: "card.tsx", icon: <FileText aria-hidden="true" /> },
        ],
      },
      { id: "index.ts", label: "index.ts", icon: <FileText aria-hidden="true" /> },
    ],
  },
  { id: "package.json", label: "package.json", icon: <FileText aria-hidden="true" /> },
];

return (
  <TreeView
    data={data}
    defaultExpandedIds={["src", "components"]}
    defaultValue="button.tsx"
    aria-label="Project files"
    className="max-w-xs rounded-lg border border-border p-1.5"
  />
);
```

</td><td>

```cronus
component TreeViewFiles layout:stack style:tree-view aria-label:"Project files" max-width:xs framed:true {
  item "src" icon:folder open:true
  item "components" icon:folder level:2 open:true
  item "button.tsx" icon:file-text level:3 selected:true
  item "card.tsx" icon:file-text level:3
  item "index.ts" icon:file-text level:2
  item "package.json" icon:file-text
}
```

</td></tr></table>

### Timeline (`/timeline`)

#### Activity

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Timeline className="max-w-md">
  <TimelineItem>
    <TimelineDot tone="primary" icon={<CreditCard aria-hidden="true" />} />
    <TimelineContent>
      <TimelineTitle>Order placed</TimelineTitle>
      <TimelineTime dateTime="2026-06-21T09:24">Jun 21 · 09:24</TimelineTime>
      <TimelineDescription>Order #4827 created for 3 items.</TimelineDescription>
    </TimelineContent>
  </TimelineItem>
  <TimelineItem>
    <TimelineDot tone="success" icon={<CheckCircle2 aria-hidden="true" />} />
    <TimelineContent>
      <TimelineTitle>Payment confirmed</TimelineTitle>
      <TimelineTime dateTime="2026-06-21T09:25">Jun 21 · 09:25</TimelineTime>
      <TimelineDescription>R$ 248,90 captured on the card ending 4242.</TimelineDescription>
    </TimelineContent>
  </TimelineItem>
  <TimelineItem>
    <TimelineDot icon={<Package aria-hidden="true" />} />
    <TimelineContent>
      <TimelineTitle>Packed</TimelineTitle>
      <TimelineTime dateTime="2026-06-22T14:02">Jun 22 · 14:02</TimelineTime>
      <TimelineDescription>Left the warehouse with the carrier.</TimelineDescription>
    </TimelineContent>
  </TimelineItem>
  <TimelineItem>
    <TimelineDot icon={<Truck aria-hidden="true" />} />
    <TimelineContent>
      <TimelineTitle>Out for delivery</TimelineTitle>
      <TimelineTime dateTime="2026-06-23T08:10">Jun 23 · 08:10</TimelineTime>
      <TimelineDescription>Arriving today between 9am and 1pm.</TimelineDescription>
    </TimelineContent>
  </TimelineItem>
</Timeline>
```

</td><td>

```cronus
component TimelineActivity layout:stack style:timeline max-width:md {
  item "Order placed" tone:primary icon:credit-card time:"Jun 21 · 09:24" datetime:"2026-06-21T09:24" description:"Order #4827 created for 3 items."
  item "Payment confirmed" tone:success icon:check-circle-2 time:"Jun 21 · 09:25" datetime:"2026-06-21T09:25" description:"R$ 248,90 captured on the card ending 4242."
  item "Packed" icon:package time:"Jun 22 · 14:02" datetime:"2026-06-22T14:02" description:"Left the warehouse with the carrier."
  item "Out for delivery" icon:truck time:"Jun 23 · 08:10" datetime:"2026-06-23T08:10" description:"Arriving today between 9am and 1pm."
}
```

</td></tr></table>

### Kanban (`/kanban`)

#### Board

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const INITIAL_BOARD: KanbanColumn[] = [
  {
    id: "todo",
    title: "To do",
    items: [
      { id: "t-1", title: "Draft the launch post", description: "Outline the key talking points." },
      { id: "t-2", title: "Audit onboarding copy" },
      {
        id: "t-3",
        title: "Collect customer quotes",
        description: "Reach out to 3 design partners.",
      },
    ],
  },
  {
    id: "in-progress",
    title: "In progress",
    items: [
      {
        id: "p-1",
        title: "Wire up the billing page",
        description: "Hook the plan picker to checkout.",
      },
      { id: "p-2", title: "Polish empty states" },
    ],
  },
  {
    id: "done",
    title: "Done",
    items: [{ id: "d-1", title: "Ship the new docs theme", description: "Dark mode shipped." }],
  },
];

function KanbanDemo() {
  const [columns, setColumns] = useState<KanbanColumn[]>(INITIAL_BOARD);
  return <Kanban columns={columns} onColumnsChange={setColumns} aria-label="Project board" />;
}
```

</td><td>

```cronus
component KanbanBoard layout:stack style:kanban aria-label:"Project board" {
  columns "To do"
  item "Draft the launch post" description:"Outline the key talking points."
  item "Audit onboarding copy"
  item "Collect customer quotes" description:"Reach out to 3 design partners."
  columns "In progress"
  item "Wire up the billing page" description:"Hook the plan picker to checkout."
  item "Polish empty states"
  columns "Done"
  item "Ship the new docs theme" description:"Dark mode shipped."
}
```

</td></tr></table>

### TodoItem (`/todo-item`)

#### Interactive

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<TodoItem
  id="1"
  title="Complete project documentation"
  description="Write comprehensive docs for the new API"
  completed={false}
  priority="high"
  dueDate={new Date()}
  project={{ id: "cronus", name: "Cronus UI" }}
  labels={[{ id: "docs", name: "Documentation" }]}
  subtasks={[
    { id: "s1", title: "Outline", completed: true },
    { id: "s2", title: "Draft", completed: false },
  ]}
  onToggleComplete={(id, completed) => console.log(id, completed)}
/>
```

</td><td>

```cronus
component Docs layout:stack style:todo-item+high due:today project:"Cronus UI" {
  title "Complete project documentation"
  text "Write comprehensive docs for the new API"
  badge "Documentation"
  badge "API"
  item "Outline" completed:true
  item "Draft"
}
```

</td></tr></table>

#### Priority levels

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex w-full max-w-lg flex-col gap-3">
  <TodoItem id="h" title="Fix critical bug in production" completed={false} priority="high" />
  <TodoItem id="m" title="Update documentation" completed={false} priority="medium" />
  <TodoItem id="l" title="Refactor legacy code" completed={false} priority="low" />
  <TodoItem id="n" title="Review team submissions" completed={false} priority="none" />
</div>
```

</td><td>

```cronus
component TodoHigh layout:stack style:todo-item+high { title "Fix critical bug in production" }
component TodoMedium layout:stack style:todo-item+medium { title "Update documentation" }
component TodoLow layout:stack style:todo-item+low { title "Refactor legacy code" }
component TodoNone layout:stack style:todo-item+none { title "Review team submissions" }
```

</td></tr></table>

### JsonViewer (`/json-viewer`)

#### API response

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const payload = {
  order: {
    id: "ord_8kX2",
    total: 248.9,
    currency: "BRL",
    paid: true,
    coupon: null,
    items: [
      { sku: "tee-black-m", qty: 2 },
      { sku: "sticker-pack", qty: 1 },
    ],
  },
};

return <JsonViewer data={payload} defaultExpandedDepth={2} />;
```

</td><td>

```cronus
component JsonViewerResponse layout:stack style:json-viewer depth:2 {
  item "order" type:object
  item "id" value:ord_8kX2 level:2
  item "total" value:248.9 level:2
  item "currency" value:BRL level:2
  item "paid" value:true level:2
  item "coupon" value:null level:2
  item "items" type:array level:2
  item "0" type:object level:3
  item "sku" value:tee-black-m level:4
  item "qty" value:2 level:4
  item "1" type:object level:3
  item "sku" value:sticker-pack level:4
  item "qty" value:1 level:4
}
```

</td></tr></table>

#### Expanded depth

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<JsonViewer data={payload} defaultExpandedDepth={Number.POSITIVE_INFINITY} />
```

</td><td>

```cronus
component JsonViewerExpanded layout:stack style:json-viewer depth:all {
  item "order" type:object
  item "id" value:ord_8kX2 level:2
  item "total" value:248.9 level:2
  item "currency" value:BRL level:2
  item "paid" value:true level:2
  item "coupon" value:null level:2
  item "items" type:array level:2
  item "0" type:object level:3
  item "sku" value:tee-black-m level:4
  item "qty" value:2 level:4
  item "1" type:object level:3
  item "sku" value:sticker-pack level:4
  item "qty" value:1 level:4
}
```

</td></tr></table>

### StatusDot (`/status-dot`)

#### Statuses

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex flex-wrap items-center gap-x-6 gap-y-3">
  <StatusDot status="online" withLabel />
  <StatusDot status="away" withLabel />
  <StatusDot status="busy" withLabel />
  <StatusDot status="offline" withLabel />
  <StatusDot status="info" withLabel label="Deploying" />
</div>
```

</td><td>

```cronus
component DotOnline layout:inline style:status-dot+online with-label:true { label "Online" }
component DotAway layout:inline style:status-dot+away with-label:true { label "Away" }
component DotBusy layout:inline style:status-dot+busy with-label:true { label "Busy" }
component DotOffline layout:inline style:status-dot+offline with-label:true { label "Offline" }
component DotDeploying layout:inline style:status-dot+info with-label:true { label "Deploying" }
```

</td></tr></table>

#### On an avatar

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function StatusDotAvatarDemo() {
  return (
    <div className="flex items-center gap-6">
      <span className="relative inline-flex">
        <Avatar>
          <AvatarImage src="https://github.com/shadcn.png" alt="@shadcn" />
          <AvatarFallback>CN</AvatarFallback>
        </Avatar>
        <StatusDot status="online" position="bottom-right" ring aria-label="@shadcn is online" />
      </span>
      <span className="relative inline-flex">
        <Avatar>
          <AvatarFallback>AL</AvatarFallback>
        </Avatar>
        <StatusDot status="busy" position="bottom-right" ring aria-label="Ada is busy" />
      </span>
      <span className="relative inline-flex">
        <Avatar>
          <AvatarFallback>JL</AvatarFallback>
        </Avatar>
        <StatusDot status="offline" position="bottom-right" ring aria-label="Jean is offline" />
      </span>
    </div>
  );
}
```

</td><td>

```cronus
component DotShadcnOnline layout:inline style:status-dot+online position:bottom-right ring:true aria-label:"@shadcn is online" avatar:"CN" src:"https://github.com/shadcn.png" alt:"@shadcn" { label "Online" }
component DotAdaBusy layout:inline style:status-dot+busy position:bottom-right ring:true aria-label:"Ada is busy" avatar:"AL" { label "Busy" }
component DotJeanOffline layout:inline style:status-dot+offline position:bottom-right ring:true aria-label:"Jean is offline" avatar:"JL" { label "Offline" }
```

</td></tr></table>

#### Pulse & sizes

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex flex-col items-center gap-5">
  <div className="flex items-center gap-6">
    <StatusDot status="error" pulse withLabel label="Live" />
    <StatusDot status="success" pulse withLabel label="Streaming" />
  </div>
  <div className="flex items-center gap-4">
    <StatusDot size="xs" aria-label="Online (xs)" />
    <StatusDot size="sm" aria-label="Online (sm)" />
    <StatusDot size="md" aria-label="Online (md)" />
    <StatusDot size="lg" aria-label="Online (lg)" />
  </div>
</div>
```

</td><td>

```cronus
component DotLive layout:inline style:status-dot+error pulse:true with-label:true { label "Live" }
component DotStreaming layout:inline style:status-dot+success pulse:true with-label:true { label "Streaming" }
component DotExtraSmall layout:inline style:status-dot+xs aria-label:"Online (xs)" { label "Online" }
component DotSmall layout:inline style:status-dot+sm aria-label:"Online (sm)" { label "Online" }
component DotMedium layout:inline style:status-dot+md aria-label:"Online (md)" { label "Online" }
component DotLarge layout:inline style:status-dot+lg aria-label:"Online (lg)" { label "Online" }
```

</td></tr></table>

### ImageZoom (`/image-zoom`)

#### Hover to zoom

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<ImageZoom
  src="https://picsum.photos/seed/cronus-product/900/600"
  alt="Studio product photo"
  className="max-w-md"
/>
```

</td><td>

```cronus
component ImageZoomProduct layout:stack style:image-zoom src:"https://picsum.photos/seed/cronus-product/900/600" alt:"Studio product photo" max-width:md {}
```

</td></tr></table>

#### Zoom scale

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="grid w-full max-w-2xl gap-4 sm:grid-cols-2">
  <div className="flex flex-col items-center gap-2">
    <ImageZoom src="https://picsum.photos/seed/cronus-loft/700/500" alt="" />
    <span className="text-sm text-fg-secondary">zoom 2 (default)</span>
  </div>
  <div className="flex flex-col items-center gap-2">
    <ImageZoom
      src="https://picsum.photos/seed/cronus-loft/700/500"
      alt=""
      zoom={4}
      labels={{ zoom: "Zoom image (4×)" }}
    />
    <span className="text-sm text-fg-secondary">zoom 4</span>
  </div>
</div>
```

</td><td>

```cronus
component ImageZoomTwo layout:stack style:image-zoom src:"https://picsum.photos/seed/cronus-loft/700/500" caption:"zoom 2 (default)" {}
component ImageZoomFour layout:stack style:image-zoom src:"https://picsum.photos/seed/cronus-loft/700/500" zoom:4 label:"Zoom image (4×)" caption:"zoom 4" {}
```

</td></tr></table>

#### Custom image & zoom state

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function ImageZoomStateDemo() {
  const [zoomed, setZoomed] = useState(false);
  return (
    <div className="flex w-full max-w-md flex-col items-center gap-3">
      <ImageZoom zoom={3} onZoomChange={setZoomed} labels={{ zoom: "Zoom fabric detail" }}>
        <img src="https://picsum.photos/seed/cronus-fabric/1200/675" alt="" />
      </ImageZoom>
      <Badge variant={zoomed ? "primary" : "secondary"}>
        {zoomed ? "Zoomed 3×" : "Hover, tap or press Enter to zoom"}
      </Badge>
    </div>
  );
}
```

</td><td>

```cronus
component ImageZoomFabric layout:stack style:image-zoom src:"https://picsum.photos/seed/cronus-fabric/1200/675" zoom:3 label:"Zoom fabric detail" badge:"Hover, tap or press Enter to zoom" max-width:md {}
```

</td></tr></table>

### VideoPlayer (`/video-player`)

#### Default

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<VideoPlayer
  src="https://media.w3.org/2010/05/sintel/trailer.mp4"
  poster="https://media.w3.org/2010/05/sintel/poster.png"
>
  <track
    kind="captions"
    src="data:text/vtt,WEBVTT%0A%0A00:00.000%20--%3E%2000:04.000%0AWind%20howls%20across%20a%20snowy%20mountain%20pass."
    srcLang="en"
    label="English"
    default
  />
</VideoPlayer>
```

</td><td>

```cronus
component VideoPlayerTrailer layout:stack style:video-player src:"https://media.w3.org/2010/05/sintel/trailer.mp4" poster:"https://media.w3.org/2010/05/sintel/poster.png" captions:"data:text/vtt,WEBVTT%0A%0A00:00.000%20--%3E%2000:04.000%0AWind%20howls%20across%20a%20snowy%20mountain%20pass." captions-lang:en captions-label:English {}
```

</td></tr></table>

#### Aspect ratios

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex w-full flex-col gap-4">
  <VideoPlayer
    src="https://media.w3.org/2010/05/sintel/trailer.mp4"
    poster="https://media.w3.org/2010/05/sintel/poster.png"
    aspect="wide"
  />
  <VideoPlayer
    src="https://media.w3.org/2010/05/sintel/trailer.mp4"
    poster="https://media.w3.org/2010/05/sintel/poster.png"
    aspect="square"
    className="mx-auto max-w-xs"
  />
</div>
```

</td><td>

```cronus
component VideoPlayerWide layout:stack style:video-player+wide src:"https://media.w3.org/2010/05/sintel/trailer.mp4" poster:"https://media.w3.org/2010/05/sintel/poster.png" {}
component VideoPlayerSquare layout:stack style:video-player+square src:"https://media.w3.org/2010/05/sintel/trailer.mp4" poster:"https://media.w3.org/2010/05/sintel/poster.png" max-width:xs {}
```

</td></tr></table>

#### Localized labels

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<VideoPlayer
  src="https://media.w3.org/2010/05/sintel/trailer.mp4"
  poster="https://media.w3.org/2010/05/sintel/poster.png"
  muted
  loop
  labels={{
    play: "Reproduzir",
    pause: "Pausar",
    mute: "Silenciar",
    unmute: "Ativar som",
    seek: "Avançar",
    volume: "Volume",
    settings: "Velocidade",
    fullscreen: "Tela cheia",
  }}
/>
```

</td><td>

```cronus
component VideoPlayerLocalized layout:stack style:video-player src:"https://media.w3.org/2010/05/sintel/trailer.mp4" poster:"https://media.w3.org/2010/05/sintel/poster.png" muted:true loop:true label-play:"Reproduzir" label-pause:"Pausar" label-mute:"Silenciar" label-unmute:"Ativar som" label-seek:"Avançar" label-volume:"Volume" label-settings:"Velocidade" label-fullscreen:"Tela cheia" {}
```

</td></tr></table>

### DescriptionList (`/description-list`)

#### Stacked

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<DescriptionList className="max-w-sm">
  <DescriptionTerm>Full name</DescriptionTerm>
  <DescriptionDetails>Margot Foster</DescriptionDetails>
  <DescriptionTerm>Email</DescriptionTerm>
  <DescriptionDetails>margot@example.com</DescriptionDetails>
  <DescriptionTerm>Plan</DescriptionTerm>
  <DescriptionDetails>Pro — billed yearly</DescriptionDetails>
</DescriptionList>
```

</td><td>

```cronus
component DescriptionStacked layout:stack style:description-list raw:true max-width:sm {
  text "Full name"
  text "Margot Foster"
  text "Email"
  text "margot@example.com"
  text "Plan"
  text "Pro — billed yearly"
}
```

</td></tr></table>

#### Horizontal order summary

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<DescriptionList layout="horizontal" detailsAlign="end" bordered className="max-w-md">
  <DescriptionItem term="Order">#10245</DescriptionItem>
  <DescriptionItem term="Date">June 23, 2026</DescriptionItem>
  <DescriptionItem term="Payment method">Visa ending in 4242</DescriptionItem>
  <DescriptionItem term="Status">
    <Badge variant="success">Paid</Badge>
  </DescriptionItem>
  <DescriptionItem term="Total">
    <span className="font-mono font-medium tabular-nums">$149.00</span>
  </DescriptionItem>
</DescriptionList>
```

</td><td>

```cronus
component DescriptionHorizontal layout:stack style:description-list+horizontal details-align:end bordered:true max-width:md {
  item "Order" details:"#10245"
  item "Date" details:"June 23, 2026"
  item "Payment method" details:"Visa ending in 4242"
  item "Status" details:"Paid" badge:success
  item "Total" details:"$149.00" font:mono
}
```

</td></tr></table>

#### Striped rows

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<DescriptionList layout="horizontal" striped className="max-w-md">
  <DescriptionItem term="Environment">Production</DescriptionItem>
  <DescriptionItem term="API version">2026-06-01</DescriptionItem>
  <DescriptionItem term="Webhook endpoint">https://api.example.com/hooks</DescriptionItem>
  <DescriptionItem term="Signing secret">whsec_••••••••</DescriptionItem>
  <DescriptionItem term="Mode">
    <Badge variant="secondary">Live</Badge>
  </DescriptionItem>
</DescriptionList>
```

</td><td>

```cronus
component DescriptionStriped layout:stack style:description-list+horizontal striped:true max-width:md {
  item "Environment" details:"Production"
  item "API version" details:"2026-06-01"
  item "Webhook endpoint" details:"https://api.example.com/hooks"
  item "Signing secret" details:"whsec_••••••••"
  item "Mode" details:"Live" badge:secondary
}
```

</td></tr></table>

#### Grid cards

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<DescriptionList layout="grid" size="sm">
  <DescriptionItem term="Region">São Paulo (GRU)</DescriptionItem>
  <DescriptionItem term="Runtime">Node 22 LTS</DescriptionItem>
  <DescriptionItem term="Instances">3 × shared-cpu</DescriptionItem>
  <DescriptionItem term="Custom domain">store.cronus.dev</DescriptionItem>
  <DescriptionItem term="TLS">Auto-managed</DescriptionItem>
  <DescriptionItem term="Deploy hook">Enabled</DescriptionItem>
</DescriptionList>
```

</td><td>

```cronus
component DescriptionGrid layout:stack style:description-list+grid+sm {
  item "Region" details:"São Paulo (GRU)"
  item "Runtime" details:"Node 22 LTS"
  item "Instances" details:"3 × shared-cpu"
  item "Custom domain" details:"store.cronus.dev"
  item "TLS" details:"Auto-managed"
  item "Deploy hook" details:"Enabled"
}
```

</td></tr></table>

## Feedback

### Alert (`/alert`)

#### Default

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Alert>
  <Terminal aria-hidden="true" />
  <AlertTitle>Heads up!</AlertTitle>
  <AlertDescription>
    You can add components to your app using the CLI.
  </AlertDescription>
</Alert>
```

</td><td>

```cronus
component HeadsUp layout:stack style:alert icon:terminal description:"You can add components to your app using the CLI." { label "Heads up!" }
```

</td></tr></table>

#### Variants

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex flex-col gap-4">
  <Alert variant="info">
    <Info aria-hidden="true" />
    <AlertTitle>New version available</AlertTitle>
    <AlertDescription>A new release is ready to install.</AlertDescription>
  </Alert>
  <Alert variant="success">
    <CircleCheck aria-hidden="true" />
    <AlertTitle>Payment received</AlertTitle>
    <AlertDescription>Your subscription is now active.</AlertDescription>
  </Alert>
  <Alert variant="warning">
    <TriangleAlert aria-hidden="true" />
    <AlertTitle>Storage almost full</AlertTitle>
    <AlertDescription>You've used 92% of your quota.</AlertDescription>
  </Alert>
  <Alert variant="destructive">
    <CircleAlert aria-hidden="true" />
    <AlertTitle>Unable to save changes</AlertTitle>
    <AlertDescription>Check your connection and try again.</AlertDescription>
  </Alert>
</div>
```

</td><td>

```cronus
component NewVersion layout:stack style:alert+info icon:info description:"A new release is ready to install." { label "New version available" }
component PaymentReceived layout:stack style:alert+success icon:circle-check description:"Your subscription is now active." { label "Payment received" }
component StorageFull layout:stack style:alert+warning icon:triangle-alert description:"You've used 92% of your quota." { label "Storage almost full" }
component UnableToSave layout:stack style:alert+destructive icon:circle-alert description:"Check your connection and try again." { label "Unable to save changes" }
```

</td></tr></table>

#### Title only

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Alert variant="info">
  <AlertTitle>Your trial ends in 3 days.</AlertTitle>
</Alert>
```

</td><td>

```cronus
component TrialEnds layout:stack style:alert+info { label "Your trial ends in 3 days." }
```

</td></tr></table>

### Banner (`/banner`)

#### Brand promo with a CTA

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Banner
  variant="brand"
  icon={<Sparkles aria-hidden="true" />}
  title="Cronus UI 1.0 is here."
  description="60+ themeable components, now stable."
  action={
    <Button size="sm" variant="secondary">
      Read the announcement
      <ArrowRight aria-hidden="true" />
    </Button>
  }
/>
```

</td><td>

```cronus
component Promo layout:stack style:banner+brand icon:sparkles description:"60+ themeable components, now stable." {
  label "Cronus UI 1.0 is here."
  action "Read the announcement" icon-end:arrow-right
}
```

</td></tr></table>

#### Dismissible

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function BannerDemo() {
  const [open, setOpen] = useState(true);

  return (
    <div className="flex w-full max-w-2xl flex-col gap-3">
      <Banner
        variant="info"
        open={open}
        onDismiss={() => setOpen(false)}
        icon={<Info aria-hidden="true" />}
        title="Scheduled maintenance"
        description="Dashboards may be briefly unavailable on Sunday at 02:00 UTC."
      />
      {open ? null : (
        <Button variant="outline" size="sm" className="self-start" onClick={() => setOpen(true)}>
          Show banner again
        </Button>
      )}
    </div>
  );
}
```

</td><td>

```cronus
component Maintenance layout:stack style:banner+info icon:info description:"Dashboards may be briefly unavailable on Sunday at 02:00 UTC." restore:"Show banner again" { label "Scheduled maintenance" }
```

</td></tr></table>

### Spinner (`/spinner`)

#### Sizes

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex items-center gap-6">
  <Spinner size="sm" aria-label="Loading small" />
  <Spinner size="md" aria-label="Loading medium" />
  <Spinner size="lg" aria-label="Loading large" />
</div>
```

</td><td>

```cronus
component SpinnerSmall layout:inline style:spinner+sm aria-label:"Loading small" { label "Loading" }
component SpinnerMedium layout:inline style:spinner+md aria-label:"Loading medium" { label "Loading" }
component SpinnerLarge layout:inline style:spinner+lg aria-label:"Loading large" { label "Loading" }
```

</td></tr></table>

### Progress (`/progress`)

#### Determinate

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function ProgressDemo() {
  const [animated, setAnimated] = useState(0);

  useEffect(() => {
    const timer = setTimeout(() => setAnimated(66), 400);
    return () => clearTimeout(timer);
  }, []);

  return (
    <div className="flex w-full max-w-sm flex-col gap-6">
      <Progress value={30} aria-label="Storage used" />
      <Progress value={animated} aria-label="Upload progress" />
      <Progress value={100} aria-label="Sync complete" />
    </div>
  );
}
```

</td><td>

```cronus
component Storage layout:stack style:progress value:30 aria-label:"Storage used" { label "Storage used" }
component Upload layout:stack style:progress value:66 animate:true aria-label:"Upload progress" { label "Upload progress" }
component Sync layout:stack style:progress value:100 aria-label:"Sync complete" { label "Sync complete" }
```

</td></tr></table>

### UsageMeter (`/usage-meter`)

#### Linear

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex w-full max-w-sm flex-col gap-6">
  <UsageMeter label="Tokens" value={186_400} max={250_000} unit="tokens" />
  <UsageMeter label="Seats" value={18} max={20} />
  <UsageMeter label="Storage" value={48} max={50} unit="GB" />
</div>
```

</td><td>

```cronus
component Tokens layout:stack style:usage-meter value:186400 max:250000 unit:tokens { label "Tokens" }
component Seats layout:stack style:usage-meter value:18 max:20 { label "Seats" }
component StorageQuota layout:stack style:usage-meter value:48 max:50 unit:GB { label "Storage" }
```

</td></tr></table>

#### Circular

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex flex-wrap items-center gap-8">
  <UsageMeterCircular label="API requests" value={6_200} max={10_000} />
  <UsageMeterCircular label="Bandwidth" value={172} max={200} unit="GB" tone="warning" />
  <UsageMeterCircular label="Builds" value={95} max={100} />
</div>
```

</td><td>

```cronus
component ApiRequests layout:inline style:usage-meter+circular value:6200 max:10000 { label "API requests" }
component Bandwidth layout:inline style:usage-meter+circular value:172 max:200 unit:GB tone:warning { label "Bandwidth" }
component Builds layout:inline style:usage-meter+circular value:95 max:100 { label "Builds" }
```

</td></tr></table>

### Toast (`/sonner`)

#### Toasts

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function ToastDemo() {
  return (
    <>
      <Toaster />
      <div className="flex flex-wrap gap-2">
        <Button variant="outline" onClick={() => toast("Event has been created.")}>
          Default
        </Button>
        <Button variant="outline" onClick={() => toast.success("Changes saved successfully.")}>
          Success
        </Button>
        <Button variant="outline" onClick={() => toast.error("Something went wrong. Please try again.")}>
          Error
        </Button>
      </div>
    </>
  );
}
```

</td><td>

```cronus
component Toasts layout:inline style:sonner {
  label "Notifications"
  action "Default" description:"Event has been created."
  action "Success" type:success description:"Changes saved successfully."
  action "Error" type:error description:"Something went wrong. Please try again."
}
```

</td></tr></table>

### AlertDialog (`/alert-dialog`)

#### Confirm

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<AlertDialog>
  <AlertDialogTrigger asChild>
    <Button variant="destructive">
      <Trash2 aria-hidden="true" />
      Delete account
    </Button>
  </AlertDialogTrigger>
  <AlertDialogContent>
    <AlertDialogHeader>
      <AlertDialogTitle>Are you absolutely sure?</AlertDialogTitle>
      <AlertDialogDescription>
        This permanently deletes your account and removes all associated data. This action
        cannot be undone.
      </AlertDialogDescription>
    </AlertDialogHeader>
    <AlertDialogFooter>
      <AlertDialogCancel>Cancel</AlertDialogCancel>
      <AlertDialogAction>Yes, delete it</AlertDialogAction>
    </AlertDialogFooter>
  </AlertDialogContent>
</AlertDialog>
```

</td><td>

```cronus
component ConfirmDelete layout:inline style:alert-dialog trigger:"Delete account" trigger-variant:destructive trigger-icon:trash-2 description:"This permanently deletes your account and removes all associated data. This action cannot be undone." cancel:"Cancel" {
  label "Are you absolutely sure?"
  action "Yes, delete it"
}
```

</td></tr></table>

## Overlays & Tooltips

### Dialog (`/dialog`)

#### Basic

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Dialog>
  <DialogTrigger asChild>
    <Button>Edit profile</Button>
  </DialogTrigger>
  <DialogContent>
    <DialogHeader>
      <DialogTitle>Edit profile</DialogTitle>
      <DialogDescription>
        Update your display name. Changes are saved when you confirm.
      </DialogDescription>
    </DialogHeader>
    <div className="flex flex-col gap-4 py-2">
      <div className="flex flex-col gap-2">
        <Label htmlFor="name">Name</Label>
        <Input id="name" defaultValue="Ada Lovelace" />
      </div>
      <div className="flex flex-col gap-2">
        <Label htmlFor="username">Username</Label>
        <Input id="username" defaultValue="ada" />
      </div>
    </div>
    <DialogFooter>
      <DialogClose asChild>
        <Button variant="outline">Cancel</Button>
      </DialogClose>
      <DialogClose asChild>
        <Button>Save changes</Button>
      </DialogClose>
    </DialogFooter>
  </DialogContent>
</Dialog>
```

</td><td>

```cronus
component EditProfile layout:inline style:dialog trigger:"Edit profile" trigger-variant:primary description:"Update your display name. Changes are saved when you confirm." cancel:"Cancel" {
  label "Edit profile"
  field "Name" value:"Ada Lovelace"
  field "Username" value:"ada"
  action "Save changes"
}
```

</td></tr></table>

### Sheet (`/sheet`)

#### Sides

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
{(["top", "right", "bottom", "left"] as const).map((side) => (
  <Sheet key={side}>
    <SheetTrigger asChild>
      <Button variant="outline" className="capitalize">
        {side}
      </Button>
    </SheetTrigger>
    <SheetContent side={side}>
      <SheetHeader>
        <SheetTitle className="capitalize">{side} sheet</SheetTitle>
        <SheetDescription>
          This panel slides in from the {side} edge. Press Escape or click
          outside to dismiss it.
        </SheetDescription>
      </SheetHeader>
      <div className="flex flex-col gap-2 px-4 py-2">
        <Label htmlFor={`note-${side}`}>Quick note</Label>
        <Input id={`note-${side}`} placeholder="Type something…" />
      </div>
      <SheetFooter>
        <SheetClose asChild>
          <Button>Done</Button>
        </SheetClose>
      </SheetFooter>
    </SheetContent>
  </Sheet>
))}
```

</td><td>

```cronus
component Top layout:inline style:sheet+top trigger:"Top" description:"This panel slides in from the top edge. Press Escape or click outside to dismiss it." {
  label "Top sheet"
  field "Quick note" placeholder:"Type something…"
  action "Done"
}
component Right layout:inline style:sheet+right trigger:"Right" description:"This panel slides in from the right edge. Press Escape or click outside to dismiss it." {
  label "Right sheet"
  field "Quick note" placeholder:"Type something…"
  action "Done"
}
component Bottom layout:inline style:sheet+bottom trigger:"Bottom" description:"This panel slides in from the bottom edge. Press Escape or click outside to dismiss it." {
  label "Bottom sheet"
  field "Quick note" placeholder:"Type something…"
  action "Done"
}
component Left layout:inline style:sheet+left trigger:"Left" description:"This panel slides in from the left edge. Press Escape or click outside to dismiss it." {
  label "Left sheet"
  field "Quick note" placeholder:"Type something…"
  action "Done"
}
```

</td></tr></table>

### Drawer (`/drawer`)

#### Bottom drawer

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Drawer>
  <DrawerTrigger asChild>
    <Button variant="outline">Open drawer</Button>
  </DrawerTrigger>
  <DrawerContent>
    <div className="mx-auto w-full max-w-sm">
      <DrawerHeader>
        <DrawerTitle>Move goal</DrawerTitle>
        <DrawerDescription>Set your daily activity target.</DrawerDescription>
      </DrawerHeader>
      <div className="px-4 py-6 text-center">
        <span className="font-display text-5xl font-semibold tracking-tight text-fg">
          350
        </span>
        <p className="mt-1 text-xs uppercase tracking-wider text-fg-tertiary">
          Calories / day
        </p>
      </div>
      <DrawerFooter>
        <Button>Submit</Button>
        <DrawerClose asChild>
          <Button variant="outline">Cancel</Button>
        </DrawerClose>
      </DrawerFooter>
    </div>
  </DrawerContent>
</Drawer>
```

</td><td>

```cronus
component MoveGoal layout:inline style:drawer trigger:"Open drawer" description:"Set your daily activity target." cancel:"Cancel" {
  label "Move goal"
  value "350" description:"Calories / day"
  action "Submit"
}
```

</td></tr></table>

### Popover (`/popover`)

#### With content

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Popover>
  <PopoverTrigger asChild>
    <Button variant="outline">Dimensions</Button>
  </PopoverTrigger>
  <PopoverContent className="w-80" align="start">
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-1">
        <h4 className="text-sm font-medium text-fg">Dimensions</h4>
        <p className="text-sm text-fg-secondary">Set the dimensions for the layer.</p>
      </div>
      <div className="grid gap-3">
        <div className="grid grid-cols-3 items-center gap-3">
          <Label htmlFor="width">Width</Label>
          <Input id="width" defaultValue="100%" className="col-span-2 h-8" />
        </div>
        <div className="grid grid-cols-3 items-center gap-3">
          <Label htmlFor="height">Height</Label>
          <Input id="height" defaultValue="auto" className="col-span-2 h-8" />
        </div>
      </div>
    </div>
  </PopoverContent>
</Popover>
```

</td><td>

```cronus
component Dimensions layout:inline style:popover trigger-variant:outline width:80 align:start description:"Set the dimensions for the layer." {
  label "Dimensions"
  title "Dimensions"
  field "Width" value:"100%"
  field "Height" value:"auto"
}
```

</td></tr></table>

### NotificationCenter (`/notification-center`)

#### Inbox

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const INITIAL: NotificationItem[] = [
  {
    id: "mention",
    title: "Ada mentioned you",
    description: "“@you can you review the payout rail PR?”",
    timestamp: "2m ago",
    icon: <MessageSquare aria-hidden="true" />,
  },
  {
    id: "star",
    title: "Your release hit 1,000 stars",
    description: "cronus-ui reached a new milestone.",
    timestamp: "1h ago",
    icon: <Heart aria-hidden="true" />,
  },
  {
    id: "invite",
    title: "New teammate joined",
    description: "Grace accepted your invite to Acme.",
    timestamp: "Yesterday",
    avatar: { fallback: "GR" },
    read: true,
  },
];

function NotificationCenterDemo() {
  const [items, setItems] = useState<NotificationItem[]>(INITIAL);

  return (
    <NotificationCenter
      notifications={items}
      onMarkAllRead={() => setItems((prev) => prev.map((i) => ({ ...i, read: true })))}
      onNotificationClick={(id) =>
        setItems((prev) => prev.map((i) => (i.id === id ? { ...i, read: true } : i)))
      }
      footer={
        <Button variant="link" size="sm" className="h-auto px-0">
          View all notifications
        </Button>
      }
    />
  );
}
```

</td><td>

```cronus
component Inbox layout:inline style:notification-center mark-all-read:"Mark all read" footer:"View all notifications" {
  label "Notifications"
  item "Ada mentioned you" description:"“@you can you review the payout rail PR?”" time:"2m ago" icon:message-square
  item "Your release hit 1,000 stars" description:"cronus-ui reached a new milestone." time:"1h ago" icon:heart
  item "New teammate joined" description:"Grace accepted your invite to Acme." time:"Yesterday" avatar:"GR" read:true
}
```

</td></tr></table>

### HoverCard (`/hover-card`)

#### Profile preview

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<HoverCard>
  <HoverCardTrigger asChild>
    <Button variant="link" className="px-0">
      @cronus
    </Button>
  </HoverCardTrigger>
  <HoverCardContent className="w-72">
    <div className="flex gap-3">
      <span
        className="grid size-11 shrink-0 place-items-center rounded-full bg-surface-overlay text-fg"
        aria-hidden="true"
      >
        <Users className="size-5" />
      </span>
      <div className="flex flex-col gap-1">
        <p className="text-sm font-semibold text-fg">Cronus</p>
        <p className="text-sm text-fg-secondary">
          The token-driven design system that themes itself.
        </p>
        <div className="mt-1 flex items-center gap-1.5 text-xs text-fg-tertiary">
          <CalendarDays className="size-3.5" aria-hidden="true" />
          Joined June 2026
        </div>
      </div>
    </div>
  </HoverCardContent>
</HoverCard>
```

</td><td>

```cronus
component ProfilePreview layout:inline style:hover-card icon:users width:72 description:"The token-driven design system that themes itself." {
  label "@cronus"
  title "Cronus"
  meta "Joined June 2026" icon:calendar-days
}
```

</td></tr></table>

### AuthorTooltip (`/author-tooltip`)

#### Profile

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<AuthorTooltip
  author={{
    name: "Aryan",
    avatar: "https://github.com/aryanranderiya.png",
    role: "Founder & CEO",
    github: "https://github.com/aryanranderiya",
    twitter: "https://twitter.com/aryanranderiya",
    linkedin: "https://linkedin.com/in/aryanranderiya",
  }}
/>
```

</td><td>

```cronus
component AuthorProfile layout:inline style:author-tooltip role:"Founder & CEO" avatar:"https://github.com/aryanranderiya.png" github:"https://github.com/aryanranderiya" twitter:"https://twitter.com/aryanranderiya" linkedin:"https://linkedin.com/in/aryanranderiya" { label "Aryan" }
```

</td></tr></table>

#### Sizes

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex items-center gap-3">
  <AuthorTooltip author={author} avatarSize="sm" />
  <AuthorTooltip author={author} avatarSize="md" />
  <AuthorTooltip author={author} avatarSize="lg" />
</div>
```

</td><td>

```cronus
component AuthorSm layout:inline style:author-tooltip+sm role:"Founder & CEO" avatar:"https://github.com/aryanranderiya.png" { label "Aryan" }
component AuthorMd layout:inline style:author-tooltip+md role:"Founder & CEO" avatar:"https://github.com/aryanranderiya.png" { label "Aryan" }
component AuthorLg layout:inline style:author-tooltip+lg role:"Founder & CEO" avatar:"https://github.com/aryanranderiya.png" { label "Aryan" }
```

</td></tr></table>

### Tooltip (`/tooltip`)

#### On a button

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<TooltipProvider>
  <Tooltip>
    <TooltipTrigger asChild>
      <Button variant="outline" size="icon" aria-label="Add to library">
        <Plus aria-hidden="true" />
      </Button>
    </TooltipTrigger>
    <TooltipContent>Add to library</TooltipContent>
  </Tooltip>
  <Tooltip>
    <TooltipTrigger asChild>
      <Button variant="outline">
        <HelpCircle aria-hidden="true" />
        Need help?
      </Button>
    </TooltipTrigger>
    <TooltipContent>We usually reply within minutes.</TooltipContent>
  </Tooltip>
</TooltipProvider>
```

</td><td>

```cronus
component AddToLibrary layout:inline style:tooltip+icon icon:plus { label "Add to library" }
component NeedHelp layout:inline style:tooltip icon:help-circle {
  label "Need help?"
  text "We usually reply within minutes."
}
```

</td></tr></table>

### ComponentPreviewTooltip (`/component-preview-tooltip`)

#### Live preview

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<ComponentPreviewTooltip
  componentName="goal-card"
  preview={
    <div className="w-56 rounded-2xl border border-border bg-surface-raised p-4">
      <p className="text-sm text-fg">Launch MVP</p>
      <div className="mt-3 h-2 rounded-full bg-surface-overlay">
        <div className="h-2 w-3/4 rounded-full bg-primary" />
      </div>
    </div>
  }
>
  <Button variant="outline">Hover me: Goal Card</Button>
</ComponentPreviewTooltip>
```

</td><td>

```cronus
component GoalCardPreview layout:inline style:component-preview-tooltip name:goal-card progress:75 {
  label "Hover me: Goal Card"
  title "Launch MVP"
}
component TodoItemPreview layout:inline style:component-preview-tooltip name:todo-item {
  label "Hover me: Todo Item"
  title "Review pull requests"
  text "High · 1/2 subtasks"
}
```

</td></tr></table>

### LinkPreview (`/link-preview`)

#### Unfurl

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<p className="max-w-prose text-fg-secondary">
  Check out the{" "}
  <LinkPreview href="https://github.com/pedrogbraz/cronus-ui">
    Cronus UI repository
  </LinkPreview>{" "}
  to browse the catalog.
</p>
```

</td><td>

```cronus
component RepoLink layout:stack style:link-preview href:"https://github.com/pedrogbraz/cronus-ui" prefix:"Check out the" suffix:"to browse the catalog." site:"GitHub" favicon:"https://github.com/favicon.ico" image:"https://opengraph.githubassets.com/1/pedrogbraz/cronus-ui" {
  label "Cronus UI repository"
  title "GitHub - pedrogbraz/cronus-ui"
  text "Cronus UI: the design system behind the Cronus kernel — tokens, primitives and premium components."
}
```

</td></tr></table>

### DropdownMenu (`/dropdown-menu`)

#### Actions

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function ActionsMenu() {
  const [showStatusBar, setShowStatusBar] = useState(true);

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline">Open menu</Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-56" align="start">
        <DropdownMenuLabel>My account</DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuGroup>
          <DropdownMenuItem>
            <User aria-hidden="true" />
            Profile
            <DropdownMenuShortcut>⇧⌘P</DropdownMenuShortcut>
          </DropdownMenuItem>
          <DropdownMenuItem>
            <CreditCard aria-hidden="true" />
            Billing
            <DropdownMenuShortcut>⌘B</DropdownMenuShortcut>
          </DropdownMenuItem>
          <DropdownMenuItem>
            <Settings aria-hidden="true" />
            Settings
            <DropdownMenuShortcut>⌘S</DropdownMenuShortcut>
          </DropdownMenuItem>
        </DropdownMenuGroup>
        <DropdownMenuSeparator />
        <DropdownMenuCheckboxItem
          checked={showStatusBar}
          onCheckedChange={setShowStatusBar}
        >
          Show status bar
        </DropdownMenuCheckboxItem>
        <DropdownMenuSeparator />
        <DropdownMenuSub>
          <DropdownMenuSubTrigger>
            <Users aria-hidden="true" />
            Invite team
          </DropdownMenuSubTrigger>
          <DropdownMenuSubContent>
            <DropdownMenuItem>
              <UserPlus aria-hidden="true" />
              Email invite
            </DropdownMenuItem>
            <DropdownMenuItem>
              <Copy aria-hidden="true" />
              Copy invite link
            </DropdownMenuItem>
          </DropdownMenuSubContent>
        </DropdownMenuSub>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
```

</td><td>

```cronus
component Actions layout:inline style:dropdown-menu trigger-variant:outline width:56 {
  label "Open menu"
  item "My account" type:label
  item "-"
  item "Profile" icon:user shortcut:"⇧⌘P"
  item "Billing" icon:credit-card shortcut:"⌘B"
  item "Settings" icon:settings shortcut:"⌘S"
  item "-"
  item "Show status bar" type:checkbox checked:true
  item "-"
  item "Invite team" icon:users
  item "Email invite" icon:user-plus sub:"Invite team"
  item "Copy invite link" icon:copy sub:"Invite team"
}
```

</td></tr></table>

### ContextMenu (`/context-menu`)

#### On a surface

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function ContextMenuDemo() {
  const [bookmarked, setBookmarked] = useState(true);
  const [branch, setBranch] = useState("main");

  return (
    <ContextMenu>
      <ContextMenuTrigger className="flex h-36 w-full max-w-sm select-none items-center justify-center rounded-lg border border-dashed border-border text-sm text-fg-secondary">
        Right-click here
      </ContextMenuTrigger>
      <ContextMenuContent className="w-56">
        <ContextMenuItem>
          <Scissors aria-hidden="true" />
          Cut
          <ContextMenuShortcut>⌘X</ContextMenuShortcut>
        </ContextMenuItem>
        <ContextMenuItem>
          <Copy aria-hidden="true" />
          Copy
          <ContextMenuShortcut>⌘C</ContextMenuShortcut>
        </ContextMenuItem>
        <ContextMenuItem>
          <ClipboardPaste aria-hidden="true" />
          Paste
          <ContextMenuShortcut>⌘V</ContextMenuShortcut>
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuCheckboxItem checked={bookmarked} onCheckedChange={setBookmarked}>
          Bookmarked
        </ContextMenuCheckboxItem>
        <ContextMenuSub>
          <ContextMenuSubTrigger>
            <Users aria-hidden="true" />
            Share
          </ContextMenuSubTrigger>
          <ContextMenuSubContent>
            <ContextMenuItem>
              <UserPlus aria-hidden="true" />
              Invite people
            </ContextMenuItem>
            <ContextMenuItem>
              <Copy aria-hidden="true" />
              Copy link
            </ContextMenuItem>
          </ContextMenuSubContent>
        </ContextMenuSub>
        <ContextMenuSeparator />
        <ContextMenuLabel>Branch</ContextMenuLabel>
        <ContextMenuRadioGroup value={branch} onValueChange={setBranch}>
          <ContextMenuRadioItem value="main">main</ContextMenuRadioItem>
          <ContextMenuRadioItem value="develop">develop</ContextMenuRadioItem>
        </ContextMenuRadioGroup>
      </ContextMenuContent>
    </ContextMenu>
  );
}
```

</td><td>

```cronus
component Surface layout:stack style:context-menu trigger:"Right-click here" trigger-variant:area width:56 {
  label "Right-click here"
  item "Cut" icon:scissors shortcut:"⌘X"
  item "Copy" icon:copy shortcut:"⌘C"
  item "Paste" icon:clipboard-paste shortcut:"⌘V"
  item "-"
  item "Bookmarked" type:checkbox checked:true
  item "Share" icon:users
  item "Invite people" icon:user-plus sub:"Share"
  item "Copy link" icon:copy sub:"Share"
  item "-"
  item "Branch" type:label
  item "main" type:radio checked:true
  item "develop" type:radio
}
```

</td></tr></table>

### Command (`/command`)

#### Command palette

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function CommandPalette() {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if (event.key === "k" && (event.metaKey || event.ctrlKey)) {
        event.preventDefault();
        setOpen((value) => !value);
      }
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, []);

  return (
    <>
      <Button variant="outline" onClick={() => setOpen(true)}>
        Open command palette
        <Badge variant="secondary" className="ml-1 font-mono text-[10px]">
          ⌘K
        </Badge>
      </Button>

      <CommandDialog
        open={open}
        onOpenChange={setOpen}
        title="Command palette"
        description="Search for a command to run."
      >
        <CommandInput placeholder="Type a command or search…" />
        <CommandList>
          <CommandEmpty>No results found.</CommandEmpty>
          <CommandGroup heading="Suggestions">
            <CommandItem onSelect={() => setOpen(false)}>
              <CalendarDays aria-hidden="true" />
              Calendar
            </CommandItem>
            <CommandItem onSelect={() => setOpen(false)}>
              <User aria-hidden="true" />
              Search profile
            </CommandItem>
            <CommandItem onSelect={() => setOpen(false)}>
              <Copy aria-hidden="true" />
              Copy link
            </CommandItem>
          </CommandGroup>
          <CommandSeparator />
          <CommandGroup heading="Settings">
            <CommandItem onSelect={() => setOpen(false)}>
              <Settings aria-hidden="true" />
              Settings
              <CommandShortcut>⌘S</CommandShortcut>
            </CommandItem>
            <CommandItem onSelect={() => setOpen(false)}>
              <Keyboard aria-hidden="true" />
              Keyboard shortcuts
              <CommandShortcut>⌘K</CommandShortcut>
            </CommandItem>
            <CommandItem onSelect={() => setOpen(false)}>
              <LifeBuoy aria-hidden="true" />
              Help &amp; support
            </CommandItem>
          </CommandGroup>
        </CommandList>
      </CommandDialog>
    </>
  );
}
```

</td><td>

```cronus
component Palette layout:inline style:command trigger:"Open command palette" trigger-variant:outline trigger-badge:"⌘K" placeholder:"Type a command or search…" empty:"No results found." description:"Search for a command to run." {
  label "Command palette"
  item "Calendar" icon:calendar-days group:"Suggestions"
  item "Search profile" icon:user group:"Suggestions"
  item "Copy link" icon:copy group:"Suggestions"
  item "Settings" icon:settings shortcut:"⌘S" group:"Settings"
  item "Keyboard shortcuts" icon:keyboard shortcut:"⌘K" group:"Settings"
  item "Help & support" icon:life-buoy group:"Settings"
}
```

</td></tr></table>

### Lightbox (`/lightbox`)

#### Gallery

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function LightboxDemo() {
  const [open, setOpen] = useState(false);
  const [index, setIndex] = useState(0);
  const images = [
    { src: "/photos/1.jpg", alt: "Mountain ridge at dawn" },
    { src: "/photos/2.jpg", alt: "Coastline from above" },
    // …
  ];

  return (
    <>
      <div className="grid grid-cols-3 gap-2">
        {images.map((image, i) => (
          <button
            key={image.src}
            type="button"
            onClick={() => {
              setIndex(i);
              setOpen(true);
            }}
          >
            <img src={image.src} alt={image.alt} />
          </button>
        ))}
      </div>
      <Lightbox
        images={images}
        open={open}
        onOpenChange={setOpen}
        index={index}
        onIndexChange={setIndex}
      />
    </>
  );
}
```

</td><td>

```cronus
component Gallery layout:stack style:lightbox open:false aria-label:"Image gallery" {
  label "Image gallery"
  text "Mountain ridge at dawn"
  text "Coastline from above"
  text "Forest trail in fog"
}
```

</td></tr></table>

### ConfirmationDialog (`/confirmation-dialog`)

#### Basic

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<ConfirmationDialog
  trigger={<Button>Publish article</Button>}
  icon={<Rocket aria-hidden="true" />}
  title="Publish this article?"
  description="It becomes visible to everyone on your blog immediately. You can unpublish it again at any time."
  confirmLabel="Publish"
/>
```

</td><td>

```cronus
component PublishArticle layout:inline style:confirmation-dialog trigger:"Publish article" trigger-variant:primary icon:rocket description:"It becomes visible to everyone on your blog immediately. You can unpublish it again at any time." confirm:"Publish" { label "Publish this article?" }
```

</td></tr></table>

#### Destructive

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<ConfirmationDialog
  destructive
  trigger={<Button variant="outline">Delete account</Button>}
  title="Delete your account?"
  description="This permanently deletes your account and everything in it. This action cannot be undone."
  confirmLabel="Delete account"
  cancelLabel="Cancel"
/>
```

</td><td>

```cronus
component DeleteAccount layout:inline style:confirmation-dialog+destructive trigger:"Delete account" description:"This permanently deletes your account and everything in it. This action cannot be undone." confirm:"Delete account" cancel:"Cancel" { label "Delete your account?" }
```

</td></tr></table>

#### Async confirm

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function ConfirmationDialogAsyncDemo() {
  const [open, setOpen] = useState(false);
  const attemptRef = useRef(0);
  const [deleted, setDeleted] = useState(false);

  return (
    <div className="flex flex-col items-center gap-3">
      <ConfirmationDialog
        open={open}
        onOpenChange={(next) => {
          if (next) {
            attemptRef.current = 0;
            setDeleted(false);
          }
          setOpen(next);
        }}
        destructive
        trigger={<Button variant="outline">Delete workspace</Button>}
        title="Delete this workspace?"
        description="This permanently removes 3 projects and 128 files. This action can’t be undone."
        confirmLabel="Delete workspace"
        cancelLabel="Keep workspace"
        onConfirm={() =>
          new Promise<void>((resolve, reject) => {
            attemptRef.current += 1;
            window.setTimeout(() => {
              if (attemptRef.current === 1) {
                reject(new Error("Couldn’t reach the server. Check your connection and try again."));
              } else {
                setDeleted(true);
                resolve();
              }
            }, 1400);
          })
        }
      />
      <p className="text-xs text-fg-tertiary">
        {deleted
          ? "Workspace deleted."
          : "Confirm to see the pending spinner — the first attempt fails inline, the retry succeeds."}
      </p>
    </div>
  );
}
```

</td><td>

```cronus
component DeleteWorkspace layout:inline style:confirmation-dialog+destructive trigger:"Delete workspace" description:"This permanently removes 3 projects and 128 files. This action can’t be undone." confirm:"Delete workspace" cancel:"Keep workspace" { label "Delete this workspace?" }
```

</td></tr></table>

### InviteDialog (`/invite-dialog`)

#### Basic

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<InviteDialog trigger={<Button>Invite member</Button>} />
```

</td><td>

```cronus
component Invite layout:inline style:invite-dialog trigger:"Invite member" trigger-variant:primary { label "Invite member" }
```

</td></tr></table>

#### Async invite

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function InviteDialogAsyncDemo() {
  const [open, setOpen] = useState(false);
  const attemptRef = useRef(0);
  const [invited, setInvited] = useState(false);

  return (
    <div className="flex flex-col items-center gap-3">
      <InviteDialog
        open={open}
        onOpenChange={(next) => {
          if (next) {
            attemptRef.current = 0;
            setInvited(false);
          }
          setOpen(next);
        }}
        trigger={<Button>Invite member</Button>}
        onInvite={async () => {
          attemptRef.current += 1;
          await new Promise((resolve) => {
            window.setTimeout(resolve, 1400);
          });
          if (attemptRef.current === 1) {
            throw new Error("Couldn’t reach the server. Check your connection and try again.");
          }
          setInvited(true);
        }}
      />
      <p className="text-xs text-fg-tertiary">
        {invited
          ? "Invite sent."
          : "Send to see the pending spinner — the first attempt fails inline, the retry succeeds."}
      </p>
    </div>
  );
}
```

</td><td>

```cronus
component AsyncInvite layout:inline style:invite-dialog trigger:"Invite member" trigger-variant:primary { label "Invite member" }
```

</td></tr></table>

## Menus & Navigation

### Tabs (`/tabs`)

#### Three tabs

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Tabs defaultValue="account" className="w-full max-w-md">
  <TabsList>
    <TabsTrigger value="account">Account</TabsTrigger>
    <TabsTrigger value="password">Password</TabsTrigger>
    <TabsTrigger value="team">Team</TabsTrigger>
  </TabsList>
  <TabsContent value="account" className="pt-4">
    <p className="text-sm text-fg-secondary">
      Manage your account details and public profile.
    </p>
  </TabsContent>
  <TabsContent value="password" className="pt-4">
    <p className="text-sm text-fg-secondary">
      Change your password and configure two-factor authentication.
    </p>
  </TabsContent>
  <TabsContent value="team" className="pt-4">
    <p className="text-sm text-fg-secondary">
      Invite teammates and manage their roles and permissions.
    </p>
  </TabsContent>
</Tabs>
```

</td><td>

```cronus
component Settings layout:stack style:tabs {
  label "Settings"
  tab "Account" description:"Manage your account details and public profile."
  tab "Password" description:"Change your password and configure two-factor authentication."
  tab "Team" description:"Invite teammates and manage their roles and permissions."
}
```

</td></tr></table>

### Accordion (`/accordion`)

#### FAQ

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Accordion type="single" collapsible className="w-full max-w-md">
  <AccordionItem value="item-1">
    <AccordionTrigger>Is it accessible?</AccordionTrigger>
    <AccordionContent>
      Yes. It follows the WAI-ARIA disclosure pattern and is fully keyboard
      navigable.
    </AccordionContent>
  </AccordionItem>
  <AccordionItem value="item-2">
    <AccordionTrigger>Is it themeable?</AccordionTrigger>
    <AccordionContent>
      Absolutely. Every color, radius, and shadow flows from semantic design
      tokens.
    </AccordionContent>
  </AccordionItem>
  <AccordionItem value="item-3">
    <AccordionTrigger>Is it animated?</AccordionTrigger>
    <AccordionContent>
      Yes — content expands and collapses with a smooth height transition.
    </AccordionContent>
  </AccordionItem>
</Accordion>
```

</td><td>

```cronus
component Faq layout:stack style:accordion {
  label "FAQ"
  item "Is it accessible?" description:"Yes. It follows the WAI-ARIA disclosure pattern and is fully keyboard navigable."
  item "Is it themeable?" description:"Absolutely. Every color, radius, and shadow flows from semantic design tokens."
  item "Is it animated?" description:"Yes — content expands and collapses with a smooth height transition."
}
```

</td></tr></table>

### Breadcrumb (`/breadcrumb`)

#### Trail

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Breadcrumb>
  <BreadcrumbList>
    <BreadcrumbItem>
      <BreadcrumbLink href="#">Home</BreadcrumbLink>
    </BreadcrumbItem>
    <BreadcrumbSeparator />
    <BreadcrumbItem>
      <BreadcrumbLink href="#">Components</BreadcrumbLink>
    </BreadcrumbItem>
    <BreadcrumbSeparator />
    <BreadcrumbItem>
      <BreadcrumbPage>Button</BreadcrumbPage>
    </BreadcrumbItem>
  </BreadcrumbList>
</Breadcrumb>
```

</td><td>

```cronus
component Trail layout:inline style:breadcrumb {
  label "Breadcrumb"
  item "Home" -> "#"
  item "Components" -> "#"
  item "Button"
}
```

</td></tr></table>

### Pagination (`/pagination`)

#### Pager

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Pagination>
  <PaginationContent>
    <PaginationItem>
      <PaginationPrevious href="#" />
    </PaginationItem>
    <PaginationItem>
      <PaginationLink href="#">1</PaginationLink>
    </PaginationItem>
    <PaginationItem>
      <PaginationLink href="#" isActive>
        2
      </PaginationLink>
    </PaginationItem>
    <PaginationItem>
      <PaginationLink href="#">3</PaginationLink>
    </PaginationItem>
    <PaginationItem>
      <PaginationEllipsis />
    </PaginationItem>
    <PaginationItem>
      <PaginationNext href="#" />
    </PaginationItem>
  </PaginationContent>
</Pagination>
```

</td><td>

```cronus
component PageLinks layout:stack style:pagination {
  label "Pagination"
  item "1" -> "#"
  item "2" -> "#" active:true
  item "3" -> "#"
  item "…"
}
```

</td></tr></table>

### NavigationMenu (`/navigation-menu`)

#### Menu bar

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<NavigationMenu>
  <NavigationMenuList>
    <NavigationMenuItem>
      <NavigationMenuTrigger>Products</NavigationMenuTrigger>
      <NavigationMenuContent>
        <ul className="grid w-[22rem] gap-1">
          <li>
            <NavigationMenuLink href="#">
              <span className="font-medium text-fg">Analytics</span>
              <span className="text-fg-tertiary">Real-time dashboards and reports.</span>
            </NavigationMenuLink>
          </li>
          <li>
            <NavigationMenuLink href="#">
              <span className="font-medium text-fg">Automations</span>
              <span className="text-fg-tertiary">Wire up rules and workflows.</span>
            </NavigationMenuLink>
          </li>
        </ul>
      </NavigationMenuContent>
    </NavigationMenuItem>
    <NavigationMenuItem>
      <NavigationMenuLink href="#" className={navigationMenuTriggerStyle()}>
        Docs
      </NavigationMenuLink>
    </NavigationMenuItem>
  </NavigationMenuList>
</NavigationMenu>
```

</td><td>

```cronus
component MenuBar layout:inline style:navigation-menu {
  label "Main"
  item "Products"
  link "Analytics" -> "#" description:"Real-time dashboards and reports." menu:"Products"
  link "Automations" -> "#" description:"Wire up rules and workflows." menu:"Products"
  link "Docs" -> "#"
}
```

</td></tr></table>

### Menubar (`/menubar`)

#### Menus

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function MenubarDemo() {
  const [showFullPath, setShowFullPath] = useState(true);
  const [profile, setProfile] = useState("benoit");

  return (
    <Menubar>
      <MenubarMenu>
        <MenubarTrigger>File</MenubarTrigger>
        <MenubarContent>
          <MenubarItem>
            New Tab
            <MenubarShortcut>⌘T</MenubarShortcut>
          </MenubarItem>
          <MenubarItem>
            New Window
            <MenubarShortcut>⌘N</MenubarShortcut>
          </MenubarItem>
          <MenubarSeparator />
          <MenubarSub>
            <MenubarSubTrigger>Share</MenubarSubTrigger>
            <MenubarSubContent>
              <MenubarItem>Email link</MenubarItem>
              <MenubarItem>Messages</MenubarItem>
            </MenubarSubContent>
          </MenubarSub>
          <MenubarSeparator />
          <MenubarItem>
            Print…
            <MenubarShortcut>⌘P</MenubarShortcut>
          </MenubarItem>
        </MenubarContent>
      </MenubarMenu>
      <MenubarMenu>
        <MenubarTrigger>Edit</MenubarTrigger>
        <MenubarContent>
          <MenubarItem>
            Undo
            <MenubarShortcut>⌘Z</MenubarShortcut>
          </MenubarItem>
          <MenubarItem>
            Redo
            <MenubarShortcut>⇧⌘Z</MenubarShortcut>
          </MenubarItem>
        </MenubarContent>
      </MenubarMenu>
      <MenubarMenu>
        <MenubarTrigger>View</MenubarTrigger>
        <MenubarContent>
          <MenubarCheckboxItem checked={showFullPath} onCheckedChange={setShowFullPath}>
            Always Show Full URLs
          </MenubarCheckboxItem>
          <MenubarSeparator />
          <MenubarRadioGroup value={profile} onValueChange={setProfile}>
            <MenubarRadioItem value="benoit">Benoit</MenubarRadioItem>
            <MenubarRadioItem value="evil-rabbit">Evil Rabbit</MenubarRadioItem>
          </MenubarRadioGroup>
        </MenubarContent>
      </MenubarMenu>
    </Menubar>
  );
}
```

</td><td>

```cronus
component Menus layout:inline style:menubar {
  item "New Tab" menu:"File" shortcut:"⌘T"
  item "New Window" menu:"File" shortcut:"⌘N"
  item "-" menu:"File"
  item "Share" menu:"File"
  item "Email link" menu:"File" sub:"Share"
  item "Messages" menu:"File" sub:"Share"
  item "-" menu:"File"
  item "Print…" menu:"File" shortcut:"⌘P"
  item "Undo" menu:"Edit" shortcut:"⌘Z"
  item "Redo" menu:"Edit" shortcut:"⇧⌘Z"
  item "Always Show Full URLs" menu:"View" type:checkbox checked:true
  item "-" menu:"View"
  item "Benoit" menu:"View" type:radio checked:true
  item "Evil Rabbit" menu:"View" type:radio
}
```

</td></tr></table>

### Sidebar (`/sidebar`)

#### Collapsible navigation

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<SidebarProvider className="!min-h-0 h-full">
  <Sidebar collapsible="icon" className="!h-full">
    <SidebarHeader>
      <span className="px-2 text-sm font-semibold text-fg">Acme Inc.</span>
    </SidebarHeader>
    <SidebarContent>
      <SidebarGroup>
        <SidebarGroupLabel>Workspace</SidebarGroupLabel>
        <SidebarGroupContent>
          <SidebarMenu>
            <SidebarMenuItem>
              <SidebarMenuButton isActive tooltip="Home">
                <Home />
                <span>Home</span>
              </SidebarMenuButton>
            </SidebarMenuItem>
            <SidebarMenuItem>
              <SidebarMenuButton tooltip="Inbox">
                <Inbox />
                <span>Inbox</span>
              </SidebarMenuButton>
            </SidebarMenuItem>
            <SidebarMenuItem>
              <SidebarMenuButton tooltip="Calendar">
                <Calendar />
                <span>Calendar</span>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>
    </SidebarContent>
    <SidebarFooter>
      <SidebarMenu>
        <SidebarMenuItem>
          <SidebarMenuButton tooltip="Settings">
            <Settings />
            <span>Settings</span>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarFooter>
  </Sidebar>
  <div className="flex flex-1 flex-col gap-3 p-4">
    <SidebarTrigger />
    <p className="text-sm text-fg-secondary">Toggle the sidebar with the button above.</p>
  </div>
</SidebarProvider>
```

</td><td>

```cronus
component Collapsible layout:stack style:sidebar collapsible:icon header:"Acme Inc." description:"Toggle the sidebar with the button above." {
  label "Sidebar"
  item "Home" icon:home active:true group:"Workspace"
  item "Inbox" icon:inbox group:"Workspace"
  item "Calendar" icon:calendar group:"Workspace"
  item "Settings" icon:settings footer:true
}
```

</td></tr></table>

### AppShell (`/app-shell`)

#### Shell layout

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<AppShell
  header={
    <>
      <SidebarTrigger />
      <span className="text-sm font-medium text-fg">Dashboard</span>
    </>
  }
  sidebar={
    <Sidebar collapsible="icon">
      <SidebarHeader>
        <span className="px-2 text-sm font-semibold text-fg">Acme Inc.</span>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupContent>
            <SidebarMenu>
              <SidebarMenuItem>
                <SidebarMenuButton isActive tooltip="Home">
                  <Home />
                  <span>Home</span>
                </SidebarMenuButton>
              </SidebarMenuItem>
              <SidebarMenuItem>
                <SidebarMenuButton tooltip="Search">
                  <Search />
                  <span>Search</span>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
    </Sidebar>
  }
>
  <div className="p-6 text-sm text-fg-secondary">Your page content goes here.</div>
</AppShell>
```

</td><td>

```cronus
component Shell layout:stack style:app-shell collapsible:icon header:"Acme Inc." description:"Your page content goes here." {
  label "Dashboard"
  item "Home" icon:home active:true
  item "Search" icon:search
}
```

</td></tr></table>

### WorkspaceSwitcher (`/workspace-switcher`)

#### Basic

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function WorkspaceSwitcherDemo() {
  const [workspaceId, setWorkspaceId] = useState("cronus");
  const workspaces = [
    { id: "cronus", name: "Cronus", initials: "CR" },
    { id: "northwind", name: "Northwind", initials: "NW" },
    { id: "acme", name: "Acme", initials: "AC" },
  ];

  return (
    <div className="w-56">
      <WorkspaceSwitcher
        workspaces={workspaces}
        value={workspaceId}
        onValueChange={setWorkspaceId}
      />
    </div>
  );
}
```

</td><td>

```cronus
component Basic layout:inline style:workspace-switcher width:56 {
  label "Switch workspace"
  item "Cronus" initials:"CR"
  item "Northwind" initials:"NW"
  item "Acme" initials:"AC"
}
```

</td></tr></table>

#### In a sidebar

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<SidebarProvider>
  <Sidebar collapsible="none">
    <SidebarHeader>
      <span className="px-2 text-sm font-semibold text-fg">Cronus</span>
      <WorkspaceSwitcher
        workspaces={[
          { id: "cronus", name: "Cronus", initials: "CR" },
          { id: "northwind", name: "Northwind", initials: "NW" },
          { id: "acme", name: "Acme", initials: "AC" },
        ]}
        defaultValue="cronus"
      />
    </SidebarHeader>
    <SidebarContent>
      <SidebarGroup>
        <SidebarGroupContent>
          <SidebarMenu>
            <SidebarMenuItem>
              <SidebarMenuButton isActive>
                <Home />
                <span>Home</span>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>
    </SidebarContent>
  </Sidebar>
</SidebarProvider>
```

</td><td>

```cronus
component InSidebar layout:stack style:workspace-switcher sidebar:"Cronus" {
  label "Switch workspace"
  item "Cronus" initials:"CR"
  item "Northwind" initials:"NW"
  item "Acme" initials:"AC"
  link "Home" icon:home active:true
}
```

</td></tr></table>

### Resizable (`/resizable`)

#### Split panes

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<ResizablePanelGroup
  direction="horizontal"
  className="h-48 max-w-2xl rounded-lg border border-border"
>
  <ResizablePanel defaultSize={35} minSize={20}>
    <div className="flex h-full items-center justify-center p-4 text-sm text-fg-secondary">
      Sidebar
    </div>
  </ResizablePanel>
  <ResizableHandle withHandle />
  <ResizablePanel defaultSize={65}>
    <ResizablePanelGroup direction="vertical">
      <ResizablePanel defaultSize={60}>
        <div className="flex h-full items-center justify-center p-4 text-sm text-fg-secondary">
          Content
        </div>
      </ResizablePanel>
      <ResizableHandle />
      <ResizablePanel defaultSize={40}>
        <div className="flex h-full items-center justify-center p-4 text-sm text-fg-secondary">
          Console
        </div>
      </ResizablePanel>
    </ResizablePanelGroup>
  </ResizablePanel>
</ResizablePanelGroup>
```

</td><td>

```cronus
component SplitPanes layout:stack style:resizable handle:true height:48 bordered:true {
  label "Panes"
  item "Sidebar" size:35
  item "Content" size:60 vertical:true
  item "Console" size:40 vertical:true
}
```

</td></tr></table>

### Toolbar (`/toolbar`)

#### Formatting

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function ToolbarDemo() {
  const [marks, setMarks] = useState({ bold: true, italic: false, underline: false });
  const [align, setAlign] = useState("left");

  return (
    <Toolbar aria-label="Formatting">
      <ToolbarGroup>
        <ToolbarButton aria-label="Bold" pressed={marks.bold}>
          <Bold aria-hidden="true" />
        </ToolbarButton>
        <ToolbarButton aria-label="Italic" pressed={marks.italic}>
          <Italic aria-hidden="true" />
        </ToolbarButton>
        <ToolbarButton aria-label="Underline" pressed={marks.underline}>
          <Underline aria-hidden="true" />
        </ToolbarButton>
      </ToolbarGroup>
      <ToolbarSeparator />
      <ToolbarGroup>
        <ToolbarButton aria-label="Align left" pressed={align === "left"}>
          <AlignLeft aria-hidden="true" />
        </ToolbarButton>
        <ToolbarButton aria-label="Align center" pressed={align === "center"}>
          <AlignCenter aria-hidden="true" />
        </ToolbarButton>
        <ToolbarButton aria-label="Align right" pressed={align === "right"}>
          <AlignRight aria-hidden="true" />
        </ToolbarButton>
      </ToolbarGroup>
    </Toolbar>
  );
}
```

</td><td>

```cronus
component Formatting layout:inline style:toolbar aria-label:"Formatting" {
  label "Formatting"
  item "Bold" icon:bold type:toggle pressed:true group:"marks"
  item "Italic" icon:italic type:toggle group:"marks"
  item "Underline" icon:underline type:toggle group:"marks"
  item "Align left" icon:align-left type:radio pressed:true group:"align"
  item "Align center" icon:align-center type:radio group:"align"
  item "Align right" icon:align-right type:radio group:"align"
}
```

</td></tr></table>

### TableOfContents (`/table-of-contents`)

#### Scrollspy article

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const sections = [
  { id: "overview", label: "Overview" },
  { id: "install", label: "Installation" },
  { id: "cli", label: "With the CLI", depth: 1 },
  { id: "manual", label: "Manual setup", depth: 1 },
  { id: "theming", label: "Theming" },
  { id: "api", label: "API reference" },
];

<div className="flex gap-6">
  <TableOfContents
    items={sections}
    containerId="article"
    offset={12}
    className="w-44 shrink-0 self-start"
  />
  <section
    id="article"
    aria-label="Article content"
    tabIndex={0}
    className="h-64 flex-1 overflow-y-auto rounded-xl border border-border p-5"
  >
    {sections.map((section) => (
      <section key={section.id} className="mb-6 last:mb-0 last:min-h-56">
        <h3 id={section.id} className="text-sm font-semibold text-fg">
          {section.label}
        </h3>
        <p className="mt-1.5 text-sm leading-relaxed text-fg-secondary">…</p>
      </section>
    ))}
  </section>
</div>
```

</td><td>

```cronus
component Scrollspy layout:stack style:table-of-contents article:true {
  label "On this page"
  item "Overview" description:"Cronus UI is a token-driven design system: every color, radius and shadow flows from semantic tokens, so a single preset re-themes the whole catalog."
  item "Installation" description:"Add the package to your workspace and import the stylesheet once. The components ship as plain ESM with no build step required."
  item "With the CLI" depth:1 description:"Run the init command to scaffold the tokens file and pick a preset. The CLI writes the theme and registers the fonts for you."
  item "Manual setup" depth:1 description:"Copy the tokens file into your app, import it before your own styles and set the theme attribute on the root element."
  item "Theming" description:"Switch presets and modes with two data attributes. Looks change the material language without touching the palette."
  item "API reference" description:"Every component documents its props, slots and data attributes so audits can compare geometry across implementations."
}
```

</td></tr></table>

## Date & Time

### Calendar (`/calendar`)

#### Single date

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
// Fixed initial date keeps SSR + client identical (avoids hydration drift).
const [date, setDate] = useState<Date | undefined>(() => new Date(2026, 5, 21));

return (
  <Calendar
    mode="single"
    selected={date}
    onSelect={setDate}
    defaultMonth={new Date(2026, 5, 1)}
    fixedWeeks
    className="rounded-md"
  />
);
```

</td><td>

```cronus
component SingleDate layout:stack style:calendar mode:single selected:2026-06-21 defaultMonth:2026-06-01 fixedWeeks:true { label "June 2026" }
```

</td></tr></table>

### Countdown (`/countdown`)

#### Launch countdown

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
// Target computed once per mount. Countdown is SSR-safe on its own — it
// renders a stable "--" placeholder until mounted, so a live relative
// target causes no hydration drift.
const [target] = useState(() => Date.now() + (2 * 86_400 + 14 * 3_600 + 25 * 60 + 9) * 1_000);

return <Countdown target={target} aria-label="Launch countdown" />;
```

</td><td>

```cronus
component Launch layout:inline style:countdown value:"2:14:25:09" aria-label:"Launch countdown" { label "Launch countdown" }
```

</td></tr></table>

#### Compact with completion

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const [target, setTarget] = useState(() => Date.now() + 15_000);
const [done, setDone] = useState(false);

return (
  <div className="flex items-center gap-4">
    <Countdown compact target={target} onComplete={() => setDone(true)} aria-label="Offer ends in" />
    {done ? (
      <Button
        size="sm"
        variant="outline"
        onClick={() => {
          setDone(false);
          setTarget(Date.now() + 15_000);
        }}
      >
        Restart
      </Button>
    ) : (
      <span className="text-sm text-fg-tertiary">Offer ends soon…</span>
    )}
  </div>
);
```

</td><td>

```cronus
component OfferEnds layout:inline style:countdown compact:true value:"00:00:15" aria-label:"Offer ends in" { label "Offer ends in" }
```

</td></tr></table>

### DatePicker (`/date-picker`)

#### Pick a date

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const [date, setDate] = useState<Date | undefined>();

return (
  <div className="flex flex-col gap-3">
    <DatePicker value={date} onChange={setDate} placeholder="Pick a date" />
    <span className="text-sm text-fg-tertiary">
      {date ? `Selected: ${date.toLocaleDateString("en-US")}` : "No date selected yet."}
    </span>
  </div>
);
```

</td><td>

```cronus
component PickADate layout:stack style:date-picker placeholder:"Pick a date" { label "Pick a date" }
```

</td></tr></table>

### DateRangePicker (`/date-range-picker`)

#### Date range

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const [range, setRange] = useState<DateRange | undefined>(() => ({
  from: new Date(2026, 5, 21),
  to: new Date(2026, 5, 27),
}));

return (
  <DateRangePicker
    value={range}
    onValueChange={setRange}
    numberOfMonths={1}
    aria-label="Pick a date range"
  />
);
```

</td><td>

```cronus
component Range layout:stack style:date-range-picker from:2026-06-21 to:2026-06-27 numberOfMonths:1 aria-label:"Pick a date range" { label "Pick a date range" }
```

</td></tr></table>

#### With presets

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const [range, setRange] = useState<DateRange | undefined>();

return (
  <DateRangePicker
    value={range}
    onValueChange={setRange}
    numberOfMonths={1}
    aria-label="Pick a reporting range"
    presets={[
      { label: "Last 7 days", range: { from: addDays(today, -6), to: today } },
      { label: "Last 30 days", range: { from: addDays(today, -29), to: today } },
      { label: "This week", range: { from: today, to: addDays(today, 6) } },
    ]}
  />
);
```

</td><td>

```cronus
component Presets layout:stack style:date-range-picker numberOfMonths:1 aria-label:"Pick a reporting range" {
  label "Pick a date range"
  item "Last 7 days"
  item "Last 30 days"
  item "This week"
}
```

</td></tr></table>

### Scheduler (`/scheduler`)

#### Month view

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const events: SchedulerEvent[] = [
  { id: "standup", title: "Team standup", date: new Date(2026, 5, 2), color: "primary" },
  { id: "design", title: "Design review", date: new Date(2026, 5, 9), color: "info" },
  { id: "ship", title: "v2 ship", date: new Date(2026, 5, 12), color: "success" },
  { id: "retro", title: "Sprint retro", date: new Date(2026, 5, 12), color: "warning" },
  { id: "launch", title: "Launch party", date: new Date(2026, 5, 24), color: "success" },
];

// Fixed month keeps SSR + client identical (avoids hydration drift).
const [month, setMonth] = useState<Date>(() => new Date(2026, 5, 1));

return (
  <Scheduler
    month={month}
    onMonthChange={setMonth}
    events={events}
    today={new Date(2026, 5, 1)}
    className="max-w-2xl"
  />
);
```

</td><td>

```cronus
component MonthView layout:stack style:scheduler month:2026-06 today:2026-06-01 {
  label "June 2026"
  item "Team standup" date:2026-06-02 color:primary
  item "Design review" date:2026-06-09 color:info
  item "v2 ship" date:2026-06-12 color:success
  item "Sprint retro" date:2026-06-12 color:warning
  item "Launch party" date:2026-06-24 color:success
}
```

</td></tr></table>

### TimePicker (`/time-picker`)

#### 12-hour clock

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
// Fixed initial value keeps SSR + client identical (avoids hydration drift).
const [time, setTime] = useState<TimeValue | undefined>(() => ({ hours: 9, minutes: 30 }));

return (
  <div className="flex flex-col gap-3">
    <TimePicker value={time} onValueChange={setTime} aria-label="Meeting time" />
    <span className="text-sm text-fg-tertiary">
      {time
        ? `Selected ${String(time.hours).padStart(2, "0")}:${String(time.minutes).padStart(2, "0")}`
        : "No time selected yet."}
    </span>
  </div>
);
```

</td><td>

```cronus
component TwelveHour layout:stack style:time-picker value:"09:30" aria-label:"Meeting time" { label "Select time" }
```

</td></tr></table>

#### 24-hour with seconds

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const [time, setTime] = useState<TimeValue | undefined>(() => ({
  hours: 14,
  minutes: 45,
  seconds: 0,
}));

return (
  <TimePicker
    value={time}
    onValueChange={setTime}
    hourCycle={24}
    minuteStep={5}
    showSeconds
    aria-label="Broadcast start"
  />
);
```

</td><td>

```cronus
component TwentyFourHourSeconds layout:stack style:time-picker+24 value:"14:45:00" minuteStep:5 showSeconds:true aria-label:"Broadcast start" { label "Select time" }
```

</td></tr></table>

#### Disabled

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<TimePicker defaultValue="08:00" disabled aria-label="Locked slot" />
```

</td><td>

```cronus
component LockedSlot layout:stack style:time-picker value:"08:00" disabled:true aria-label:"Locked slot" { label "Select time" }
```

</td></tr></table>

## Charts

## Premium & Brand

### GlassCard (`/glass-card`)

#### Frosted surface

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="relative overflow-hidden rounded-2xl border border-border bg-surface-inset">
  <div className="relative p-6">
    <GlassCard className="flex flex-col gap-3 p-5">
      <span className="grid size-9 place-items-center rounded-lg bg-surface-overlay text-fg">
        <Sparkles className="size-4" aria-hidden="true" />
      </span>
      <h3 className="font-display text-base font-semibold text-fg">Premium by default</h3>
      <p className="text-sm text-fg-secondary">
        Frosted blur ships out of the box — sit it on a surface, not a Midjourney wash.
      </p>
    </GlassCard>
  </div>
</div>
```

</td><td>

```cronus
component Frosted layout:stack style:glass-card icon:sparkles backdrop:inset {
  title "Premium by default"
  text "Frosted blur ships out of the box — sit it on a surface, not a Midjourney wash."
}
```

</td></tr></table>

### GradientBorder (`/gradient-border`)

#### With glow

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<GradientBorder glow innerClassName="flex flex-col gap-3 bg-surface-raised p-5">
  <h3 className="font-display text-base font-semibold text-fg">Pro plan</h3>
  <p className="text-sm text-fg-secondary">
    The hairline ring draws the eye to your highest-value surface.
  </p>
  <div className="flex items-baseline gap-1">
    <span className="font-display text-2xl font-semibold text-fg">$29</span>
    <span className="text-sm text-fg-tertiary">/ month</span>
  </div>
</GradientBorder>
```

</td><td>

```cronus
component ProPlan layout:stack style:gradient-border glow:true {
  title "Pro plan"
  text "The hairline ring draws the eye to your highest-value surface."
  value "$29" meta:"/ month"
}
```

</td></tr></table>

#### Flat

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<GradientBorder innerClassName="flex flex-col gap-3 bg-surface-raised p-5">
  <h3 className="font-display text-base font-semibold text-fg">Starter plan</h3>
  <p className="text-sm text-fg-secondary">
    The same primary hairline, kept calm and flat for secondary surfaces.
  </p>
  <div className="flex items-baseline gap-1">
    <span className="font-display text-2xl font-semibold text-fg">$0</span>
    <span className="text-sm text-fg-tertiary">/ forever</span>
  </div>
</GradientBorder>
```

</td><td>

```cronus
component StarterPlan layout:stack style:gradient-border {
  title "Starter plan"
  text "The same primary hairline, kept calm and flat for secondary surfaces."
  value "$0" meta:"/ forever"
}
```

</td></tr></table>

### GradientText (`/gradient-text`)

#### Headline

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<GradientText asChild>
  <h3 className="font-display text-5xl font-semibold leading-[1.05] tracking-tight sm:text-6xl">
    Design that themes itself
  </h3>
</GradientText>
```

</td><td>

```cronus
component GradientHeadline layout:inline style:gradient-text as:h3 size:6xl { label "Design that themes itself" }
```

</td></tr></table>

### SpotlightCard (`/spotlight-card`)

#### Hover spotlight

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<SpotlightCard className="flex flex-col gap-3 p-5">
  <span className="grid size-9 place-items-center rounded-lg bg-surface-overlay text-primary">
    <Gauge className="size-4" aria-hidden="true" />
  </span>
  <h3 className="font-display text-base font-semibold text-fg">Accessible core</h3>
  <p className="text-sm text-fg-secondary">
    Radix primitives and focus-visible rings ship on by default.
  </p>
</SpotlightCard>
```

</td><td>

```cronus
component AccessibleCore layout:stack style:spotlight-card icon:gauge icon-tone:primary {
  title "Accessible core"
  text "Radix primitives and focus-visible rings ship on by default."
}
```

</td></tr></table>

### ScrollProgress (`/scroll-progress`)

#### Reading bar & ring

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function ScrollProgressDemo() {
  const ref = useRef<HTMLElement>(null);
  return (
    <div className="flex items-start gap-4">
      {/* A named <section> is a scrollable region; keep it keyboard-focusable. */}
      <section ref={ref} tabIndex={0} aria-label="Release notes, scrollable" className="relative h-64 overflow-y-auto rounded-lg border">
        <ScrollProgress target={ref} className="sticky top-0 z-10" />
        <div className="space-y-4 p-4">
          {/* …tall content… */}
        </div>
      </section>
      <ScrollProgress variant="circle" target={ref} size={48} />
    </div>
  );
}
```

</td><td>

```cronus
component ReadingBar layout:stack style:scroll-progress target:"Release notes, scrollable" ring:48 {
  title "Release notes"
  text "Scroll this panel to advance the bar above and the ring beside it. Both read from the same container ref, so they stay perfectly in sync without tracking the page itself." repeat:12
}
```

</td></tr></table>

### ScrollNav (`/scroll-nav`)

#### Acceptance of Terms

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<ScrollNav
  title="Terms & Conditions"
  terms={[
    { id: "acceptance-of-terms", title: "Acceptance of Terms", content: <p>…</p> },
    { id: "license-agreement", title: "License Agreement", content: <p>…</p> },
  ]}
/>
```

</td><td>

```cronus
(see scroll-nav.cronus)
```

</td></tr></table>

#### Terms

```cronus
component TermsNav layout:stack style:scroll-nav viewport:"Terms, scrollable" {
  label "Terms & Conditions"
  item "Acceptance of Terms"
  text "By accessing and using this product, you agree to be bound by these terms. If you do not agree, please do not use the product."
  text "Continued use after an update means you accept the revised terms."
  item "License Agreement"
  text "The software is licensed, not sold. The license is non-exclusive, non-transferable, and may be revoked if these terms are broken."
  item "Ownership"
  text "We retain all rights, title, and interest in the product, including intellectual property. This license does not grant ownership."
  item "Updates and Support"
  text "Updates may ship automatically. Support is provided on a best-effort basis through official channels."
  item "Limitation of Liability"
  text "In no event shall we be liable for indirect, incidental, or consequential damages arising from your use of the product."
}
```

### AuroraBackground (`/aurora-background`)

#### Animated backdrop

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<AuroraBackground className="relative flex min-h-48 items-center justify-center overflow-hidden rounded-2xl">
  <div className="flex max-w-lg flex-col items-center gap-4 px-6 py-12 text-center">
    <h3 className="font-display text-4xl font-semibold leading-[1.05] tracking-tight sm:text-5xl text-fg">
      Ship something beautiful
    </h3>
    <p className="text-balance text-sm text-fg-secondary">
      Accessible, token-driven React components with premium motion baked in.
    </p>
    <Button variant="primary" size="lg">
      Get started
      <ArrowRight aria-hidden="true" />
    </Button>
  </div>
</AuroraBackground>
```

</td><td>

```cronus
component Backdrop layout:stack style:aurora-background {
  title "Ship something beautiful" size:5xl
  text "Accessible, token-driven React components with premium motion baked in."
  action "Get started" size:lg icon-end:arrow-right
}
```

</td></tr></table>

### LogoCarousel (`/logo-carousel`)

#### Hero lockup

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const logos = [
  {
    id: "stripe",
    label: "Stripe",
    node: <span className="text-5xl font-black text-[#635bff]">stripe</span>,
  },
  {
    id: "bmw",
    label: "BMW",
    node: <span className="grid size-16 place-items-center rounded-full bg-white text-sm font-black text-black">BMW</span>,
  },
  {
    id: "typescript",
    label: "TypeScript",
    node: <span className="grid size-16 place-items-center rounded-md bg-[#3178c6] text-3xl font-black text-white">TS</span>,
  },
  {
    id: "next",
    label: "Next.js",
    node: <span className="grid size-16 place-items-center rounded-full border-2 border-fg text-3xl font-semibold">N</span>,
  },
];

<section className="flex flex-col items-center gap-8 py-12 text-center">
  <div className="space-y-2">
    <p className="font-display text-2xl font-semibold text-fg">
      The best teams are already here
    </p>
    <h3 className="font-display text-6xl font-semibold leading-none text-fg">
      Join Cronus UI
    </h3>
  </div>
  <LogoCarousel items={logos} columns={3} ariaLabel="Customer logos" />
</section>
```

</td><td>

```cronus
component HeroLockup layout:stack style:logo-carousel columns:3 interval:1600 stagger:0.12 {
  label "Customer logos"
  subtitle "The best teams are already here"
  title "Join Cronus UI"
  item "Next.js"
  item "BMW"
  item "TypeScript"
  item "Stripe"
  item "Spiral"
  item "Apple"
  item "Tailwind CSS"
  item "Vercel"
}
```

</td></tr></table>

### Marquee (`/marquee`)

#### Logo ticker

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Marquee pauseOnHover speed={32} className="py-2">
  {["stripe", "vercel", "linear", "notion", "supabase", "raycast"].map((brand) => (
    <span
      key={brand}
      className="mx-8 text-2xl font-semibold tracking-tight text-fg-secondary"
    >
      {brand}
    </span>
  ))}
</Marquee>
```

</td><td>

```cronus
component LogoTicker layout:stack style:marquee+ticker speed:32 {
  label "Customer logos"
  item "stripe"
  item "vercel"
  item "linear"
  item "notion"
  item "supabase"
  item "raycast"
}
```

</td></tr></table>

#### Testimonial wall

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex flex-col gap-4">
  <Marquee pauseOnHover speed={24}>
    {testimonials.map((t) => (
      <Card key={t.name} className="mx-3 w-80 shrink-0">
        <CardContent className="flex flex-col gap-4 pt-6">
          <p className="text-sm leading-relaxed text-fg">"{t.quote}"</p>
          <div className="flex items-center gap-3">
            <span className="grid size-9 place-items-center rounded-full bg-surface-overlay text-xs font-medium text-fg-secondary">
              {t.initials}
            </span>
            <div className="text-sm">
              <p className="font-medium text-fg">{t.name}</p>
              <p className="text-fg-tertiary">{t.role}</p>
            </div>
          </div>
        </CardContent>
      </Card>
    ))}
  </Marquee>
  <Marquee pauseOnHover direction="right" speed={24}>
    {testimonials.map((t) => ( /* …same card… */ ))}
  </Marquee>
</div>
```

</td><td>

```cronus
component TestimonialsLeft layout:stack style:marquee speed:24 {
  label "Testimonials"
  item "We shipped a polished, on-brand UI in a weekend. The theming alone paid for itself." name:"Ana Ribeiro" role:"Head of Design, Northwind" initials:"AR"
  item "Every component is accessible out of the box — our axe audit went green on the first pass." name:"Marcus Lee" role:"Staff Engineer, Atlas" initials:"ML"
  item "The motion is tasteful and respects reduced-motion. It feels premium without trying hard." name:"Priya Nair" role:"Product Lead, Lumen" initials:"PN"
  item "Drop-in registry, zero lock-in. We own the code and still get updates when we want them." name:"Tomás Costa" role:"Founder, Brava" initials:"TC"
}
component TestimonialsRight layout:stack style:marquee speed:24 direction:right {
  label "Testimonials, reversed"
  item "We shipped a polished, on-brand UI in a weekend. The theming alone paid for itself." name:"Ana Ribeiro" role:"Head of Design, Northwind" initials:"AR"
  item "Every component is accessible out of the box — our axe audit went green on the first pass." name:"Marcus Lee" role:"Staff Engineer, Atlas" initials:"ML"
  item "The motion is tasteful and respects reduced-motion. It feels premium without trying hard." name:"Priya Nair" role:"Product Lead, Lumen" initials:"PN"
  item "Drop-in registry, zero lock-in. We own the code and still get updates when we want them." name:"Tomás Costa" role:"Founder, Brava" initials:"TC"
}
```

</td></tr></table>

### MorphingPopover (`/morphing-popover`)

#### Feedback

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function FeedbackPopover() {
  const [open, setOpen] = useState(false);
  const [note, setNote] = useState("");
  const [sent, setSent] = useState(false);
  const fieldId = useId();

  return (
    <MorphingPopover
      open={open}
      onOpenChange={(next) => {
        setOpen(next);
        if (next) setSent(false);
      }}
    >
      <MorphingPopoverTrigger>
        <MessageSquarePlus aria-hidden="true" className="size-4" />
        Feedback
      </MorphingPopoverTrigger>
      <MorphingPopoverContent aria-label="Send feedback" className="w-[22rem]">
        {sent ? (
          <MorphingPopoverBody className="items-center gap-1 py-10 text-center">
            <Sparkles aria-hidden="true" className="size-5 text-primary" />
            <p className="text-sm font-medium text-fg">Thanks for the note!</p>
            <p className="text-xs text-fg-tertiary">We read every message.</p>
          </MorphingPopoverBody>
        ) : (
          <form
            onSubmit={(event) => {
              event.preventDefault();
              setSent(true);
              setNote("");
              window.setTimeout(() => setOpen(false), 1200);
            }}
          >
            <label htmlFor={fieldId} className="sr-only">
              Your feedback
            </label>
            <textarea
              id={fieldId}
              value={note}
              onChange={(event) => setNote(event.target.value)}
              rows={5}
              placeholder="Add feedback"
              className="block w-full resize-none bg-transparent px-4 pt-4 text-sm text-fg outline-none placeholder:text-fg-tertiary"
            />
            <MorphingPopoverFooter className="justify-between border-t-0 px-3 pb-3 pt-1">
              <MorphingPopoverClose />
              <Button type="submit" size="sm" variant="outline" disabled={note.trim().length === 0}>
                Submit
              </Button>
            </MorphingPopoverFooter>
          </form>
        )}
      </MorphingPopoverContent>
    </MorphingPopover>
  );
}
```

</td><td>

```cronus
component Feedback layout:inline style:morphing-popover open:false icon:message-square-plus aria-label:"Send feedback" width:88 close:true {
  label "Feedback"
  field "Your feedback" placeholder:"Add feedback" rows:5
  action "Submit" variant:outline size:sm disabled:true
}
```

</td></tr></table>

#### Quick actions

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function QuickActions() {
  const [open, setOpen] = useState(false);
  const actions = [
    { id: "edit", label: "Edit", icon: Pencil },
    { id: "duplicate", label: "Duplicate", icon: Copy },
    { id: "share", label: "Share", icon: Share2 },
  ];

  return (
    <MorphingPopover open={open} onOpenChange={setOpen}>
      <MorphingPopoverTrigger>
        Actions
        <ArrowRight aria-hidden="true" className="size-4" />
      </MorphingPopoverTrigger>
      <MorphingPopoverContent aria-label="Quick actions" className="w-56">
        <MorphingPopoverBody className="gap-0.5 p-1.5">
          {actions.map(({ id, label, icon: Icon }) => (
            <MorphingPopoverButton key={id} onClick={() => setOpen(false)}>
              <Icon aria-hidden="true" />
              {label}
            </MorphingPopoverButton>
          ))}
          <div className="my-1 h-px bg-border" />
          <MorphingPopoverButton
            onClick={() => setOpen(false)}
            className="text-fg-secondary hover:text-fg"
          >
            <Trash2 aria-hidden="true" />
            Delete
          </MorphingPopoverButton>
        </MorphingPopoverBody>
      </MorphingPopoverContent>
    </MorphingPopover>
  );
}
```

</td><td>

```cronus
component QuickActions layout:inline style:morphing-popover open:false icon-end:arrow-right aria-label:"Quick actions" width:56 {
  label "Actions"
  item "Edit" icon:pencil
  item "Duplicate" icon:copy
  item "Share" icon:share-2
  item "Delete" icon:trash-2 separator:true tone:secondary
}
```

</td></tr></table>

### Shimmer (`/shimmer`)

#### Loading sheen

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex flex-col gap-3">
  <Shimmer className="h-8 w-48 rounded-lg" />
  <Shimmer className="h-4 w-full rounded-md" />
  <Shimmer className="h-4 w-3/4 rounded-md" />
  <Shimmer className="h-10 w-32 rounded-lg" />
</div>
```

</td><td>

```cronus
component LoadingSheen layout:stack style:shimmer {
  item h:8 w:48 radius:lg
  item h:4 w:full radius:md
  item h:4 w:3/4 radius:md
  item h:10 w:32 radius:lg
}
```

</td></tr></table>

### Reveal (`/reveal`)

#### Scroll reveal

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Reveal>
  <Card className="w-full">
    <CardHeader>
      <span className="mb-1 grid size-11 place-items-center rounded-xl bg-surface-overlay text-fg">
        <Sparkles className="size-5" aria-hidden="true" />
      </span>
      <CardTitle className="text-2xl">Reveal as you scroll</CardTitle>
      <CardDescription className="text-base">
        This card fades and slides into view the moment it enters the viewport. Give it room
        so the entrance is unmistakable.
      </CardDescription>
    </CardHeader>
    <CardContent className="flex flex-col gap-5 text-sm text-fg-secondary">
      <p className="leading-relaxed">
        Wrap any block — a hero, a pricing tier, a feature grid — and it arrives with intent
        instead of popping in. Reveals fire a single time, so the section settles instead of
        replaying as you scroll past.
      </p>
      <div className="grid grid-cols-3 gap-3">
        <div className="rounded-xl border border-border bg-surface-raised p-4">
          <p className="font-display text-2xl font-semibold text-fg">Fade</p>
          <p className="mt-1 text-xs text-fg-tertiary">opacity 0 → 1</p>
        </div>
        <div className="rounded-xl border border-border bg-surface-raised p-4">
          <p className="font-display text-2xl font-semibold text-fg">Slide</p>
          <p className="mt-1 text-xs text-fg-tertiary">y 24 → 0</p>
        </div>
        <div className="rounded-xl border border-border bg-surface-raised p-4">
          <p className="font-display text-2xl font-semibold text-fg">Once</p>
          <p className="mt-1 text-xs text-fg-tertiary">no replay</p>
        </div>
      </div>
    </CardContent>
  </Card>
</Reveal>
```

</td><td>

```cronus
component ScrollReveal layout:stack style:reveal card:true icon:sparkles icon-size:lg {
  title "Reveal as you scroll"
  text "This card fades and slides into view the moment it enters the viewport. Give it room so the entrance is unmistakable."
  text "Wrap any block — a hero, a pricing tier, a feature grid — and it arrives with intent instead of popping in. Reveals fire a single time, so the section settles instead of replaying as you scroll past."
  item "Fade" description:"opacity 0 → 1"
  item "Slide" description:"y 24 → 0"
  item "Once" description:"no replay"
}
```

</td></tr></table>

### AnimatedNumber (`/animated-number`)

#### Count up

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function Revenue() {
  const [total, setTotal] = useState(12480);
  return (
    <div className="flex flex-col items-center gap-4">
      <AnimatedNumber
        value={total}
        locale="pt-BR"
        formatOptions={{ style: "currency", currency: "BRL" }}
        className="font-display text-4xl font-semibold text-fg"
      />
      <Button size="sm" variant="outline" onClick={() => setTotal((t) => t + 850)}>
        Nova venda
      </Button>
    </div>
  );
}
```

</td><td>

```cronus
component CountUp layout:inline style:animated-number value:12480 locale:pt-BR currency:BRL size:4xl {
  label "Revenue"
  action "Nova venda" variant:outline size:sm
}
```

</td></tr></table>

### NumberFlow (`/number-flow`)

#### Digit flow

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<NumberFlow value={12345} className="font-display text-5xl text-fg" />
```

</td><td>

```cronus
component DigitFlow layout:inline style:number-flow value:12345 size:5xl {
  label "Digits"
  action "Shuffle format" variant:outline size:sm icon:rotate-cw
}
```

</td></tr></table>

#### Currency

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function Revenue() {
  const [value, setValue] = useState(19348.43);
  return (
    <div className="flex flex-col items-center gap-4">
      <NumberFlow
        value={value}
        prefix="$"
        format="currency"
        className="font-display text-5xl text-fg"
      />
      <Button size="sm" variant="outline" onClick={() => setValue((n) => n + 1)}>
        Update value
      </Button>
    </div>
  );
}
```

</td><td>

```cronus
component Currency layout:inline style:number-flow value:19348.43 prefix:"$" format:currency size:5xl {
  label "Revenue"
  action "Update value" variant:outline size:sm
}
```

</td></tr></table>

### Carousel (`/carousel`)

#### Slides

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Carousel className="w-full max-w-sm" opts={{ align: "start" }}>
  <CarouselContent>
    {slides.map((slide) => (
      <CarouselItem key={slide.id}>
        <div className="flex h-40 items-center justify-center rounded-xl border border-border bg-surface-raised">
          <span className="font-display text-4xl font-semibold text-fg">{slide.n}</span>
        </div>
      </CarouselItem>
    ))}
  </CarouselContent>
  <div className="mt-4 flex items-center justify-center gap-3">
    <CarouselPrevious />
    <CarouselDots />
    <CarouselNext />
  </div>
</Carousel>
```

</td><td>

```cronus
component Slides layout:stack style:carousel width:sm dots:true {
  label "Slides"
  item "1" description:"Onboarding"
  item "2" description:"Checkout"
  item "3" description:"Repasse"
  item "4" description:"Insights"
  item "5" description:"Growth"
}
```

</td></tr></table>

### SegmentedControl (`/segmented-control`)

#### Single select

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<SegmentedControl defaultValue="30d" aria-label="Período">
  <SegmentedControlItem value="7d">7 dias</SegmentedControlItem>
  <SegmentedControlItem value="30d">30 dias</SegmentedControlItem>
  <SegmentedControlItem value="12m">12 meses</SegmentedControlItem>
</SegmentedControl>
```

</td><td>

```cronus
component Period layout:inline style:segmented-control value:"30 dias" aria-label:"Período" {
  item "7 dias"
  item "30 dias"
  item "12 meses"
}
```

</td></tr></table>

### TextEffect (`/text-effect`)

#### Headline

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<TextEffect as="h3" per="char" preset="blur" className="font-display text-3xl font-semibold text-fg">
  Ship premium by default
</TextEffect>
```

</td><td>

```cronus
component EffectHeadline layout:stack style:text-effect as:h3 per:char preset:blur size:3xl { label "Ship premium by default" }
component Subline layout:stack style:text-effect per:word preset:slide delay:0.35 size:sm tone:secondary {
  label "Every surface arrives with intent."
  action "Replay" variant:outline size:sm icon:rotate-cw
}
```

</td></tr></table>

### SlideUpText (`/slide-up-text`)

#### By words

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<SlideUpText className="font-display text-3xl text-fg">
  You can just ship things.
</SlideUpText>
```

</td><td>

```cronus
component ByWords layout:inline style:slide-up-text size:3xl { label "You can just ship things." }
```

</td></tr></table>

#### By characters

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<SlideUpText split="characters" className="font-display text-3xl text-fg">
  You just can ship things.
</SlideUpText>
```

</td><td>

```cronus
component ByCharacters layout:inline style:slide-up-text split:characters size:3xl { label "You just can ship things." }
```

</td></tr></table>

#### By lines

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<SlideUpText split="lines" className="font-display text-3xl text-fg">
  First line
  Second line
  Third line
</SlideUpText>
```

</td><td>

```cronus
component ByLines layout:inline style:slide-up-text split:lines size:3xl {
  label "First line"
  text "Second line"
  text "Third line"
}
```

</td></tr></table>

#### From last

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<SlideUpText from="last" className="font-display text-3xl text-fg">
  Animation from last word
</SlideUpText>
```

</td><td>

```cronus
component FromLast layout:inline style:slide-up-text from:last size:3xl { label "Animation from last word" }
```

</td></tr></table>

### ImagesBadge (`/images-badge`)

#### Folder

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<ImagesBadge
  text="Introducing Agenforce Marketing Template"
  images={[
    "https://assets.aceternity.com/pro/agenforce-1.webp",
    "https://assets.aceternity.com/pro/agenforce-2.webp",
    "https://assets.aceternity.com/pro/agenforce-3.webp",
  ]}
/>
```

</td><td>

```cronus
component AgenforceFolder layout:inline style:images-badge {
  label "Introducing Agenforce Marketing Template"
  item "Preview 1" -> "https://assets.aceternity.com/pro/agenforce-1.webp"
  item "Preview 2" -> "https://assets.aceternity.com/pro/agenforce-2.webp"
  item "Preview 3" -> "https://assets.aceternity.com/pro/agenforce-3.webp"
}
```

</td></tr></table>

### 3D Globe (`/globe-3d`)

#### Distributed team

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Globe3D
  markers={markers}
  config={{
    atmosphereColor: "#4da6ff",
    atmosphereIntensity: 20,
    bumpScale: 5,
    autoRotateSpeed: 0.3,
  }}
/>
```

</td><td>

```cronus
component TeamGlobe layout:stack style:globe-3d {
  label "Globe"
  item "New York" lat:40.7128 lng:-74.006 -> "https://assets.aceternity.com/avatars/1.webp"
  item "London" lat:51.5074 lng:-0.1278 -> "https://assets.aceternity.com/avatars/2.webp"
  item "Tokyo" lat:35.6762 lng:139.6503 -> "https://assets.aceternity.com/avatars/3.webp"
  item "Sydney" lat:-33.8688 lng:151.2093 -> "https://assets.aceternity.com/avatars/4.webp"
  item "Paris" lat:48.8566 lng:2.3522 -> "https://assets.aceternity.com/avatars/5.webp"
  item "New Delhi" lat:28.6139 lng:77.209 -> "https://assets.aceternity.com/avatars/6.webp"
  item "Moscow" lat:55.7558 lng:37.6173 -> "https://assets.aceternity.com/avatars/7.webp"
  item "Rio de Janeiro" lat:-22.9068 lng:-43.1729 -> "https://assets.aceternity.com/avatars/8.webp"
  item "Shanghai" lat:31.2304 lng:121.4737 -> "https://assets.aceternity.com/avatars/9.webp"
  item "Dubai" lat:25.2048 lng:55.2708 -> "https://assets.aceternity.com/avatars/10.webp"
  item "Buenos Aires" lat:-34.6037 lng:-58.3816 -> "https://assets.aceternity.com/avatars/11.webp"
  item "Singapore" lat:1.3521 lng:103.8198 -> "https://assets.aceternity.com/avatars/12.webp"
  item "Seoul" lat:37.5665 lng:126.978 -> "https://assets.aceternity.com/avatars/13.webp"
}
```

</td></tr></table>

### GlobeWireframe (`/globe-wireframe`)

#### Wireframe solid

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<GlobeWireframe
  variant="wireframesolid"
  autoRotate
  autoRotateSpeed={0.45}
  strokeWidth={0.6}
/>
```

</td><td>

```cronus
component WireframeSolidGlobe layout:stack style:globe-wireframe+wireframesolid auto-rotate:true { label "Globe" }
```

</td></tr></table>

### Frame (`/frame`)

#### Browser chrome

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Frame url="cronus.app/dashboard">
  <div className="space-y-2 p-6">
    <h3 className="font-display text-lg font-semibold text-fg">Faturamento</h3>
    <p className="text-sm text-fg-secondary">R$ 128.940 nos últimos 30 dias.</p>
    <p className="text-sm text-fg-secondary">+18% vs. o período anterior.</p>
  </div>
</Frame>
```

</td><td>

```cronus
component Browser layout:stack style:frame url:"cronus.app/dashboard" {
  title "Faturamento"
  text "R$ 128.940 nos últimos 30 dias."
  text "+18% vs. o período anterior."
}
```

</td></tr></table>

#### Window chrome

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Frame variant="window">
  <div className="space-y-2 p-6">
    <h3 className="font-display text-lg font-semibold text-fg">Preferências</h3>
    <p className="text-sm text-fg-secondary">Tema, notificações e atalhos.</p>
  </div>
</Frame>
```

</td><td>

```cronus
component Window layout:stack style:frame+window {
  title "Preferências"
  text "Tema, notificações e atalhos."
}
```

</td></tr></table>

### Dock (`/dock`)

#### App dock

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Dock
  items={[
    { icon: <Home />, label: "Home" },
    { icon: <Search />, label: "Search" },
    { icon: <Bell />, label: "Notifications" },
    { icon: <User />, label: "Profile" },
    { icon: <Settings />, label: "Settings" },
  ]}
/>
```

</td><td>

```cronus
component AppDock layout:inline style:dock {
  item "Home" icon:home
  item "Search" icon:search
  item "Notifications" icon:bell
  item "Profile" icon:user
  item "Settings" icon:settings
}
```

</td></tr></table>

### BorderBeam (`/border-beam`)

#### Featured card

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<BorderBeam duration={6} className="w-full max-w-xs">
  <div className="flex flex-col gap-4 rounded-2xl bg-surface-raised p-6">
    <div className="flex items-center justify-between">
      <span className="grid size-9 place-items-center rounded-lg bg-surface-overlay text-fg">
        <Sparkles className="size-4" aria-hidden="true" />
      </span>
      <span className="rounded-full border border-border px-2.5 py-0.5 text-xs font-medium text-fg-secondary">
        Popular
      </span>
    </div>
    <div>
      <h3 className="font-display text-base font-semibold text-fg">Pro</h3>
      <p className="mt-1 text-sm text-fg-secondary">Tudo para escalar a sua loja.</p>
    </div>
    <div className="flex items-baseline gap-1">
      <span className="font-display text-3xl font-semibold text-fg">R$ 79</span>
      <span className="text-sm text-fg-tertiary">/ mês</span>
    </div>
    <Button variant="primary" className="w-full">
      Assinar o Pro
      <ArrowRight aria-hidden="true" className="size-4" />
    </Button>
  </div>
</BorderBeam>
```

</td><td>

```cronus
component FeaturedCard layout:stack style:border-beam duration:6 width:xs icon:sparkles {
  badge "Popular"
  title "Pro"
  text "Tudo para escalar a sua loja." mt:1
  value "R$ 79" meta:"/ mês" size:3xl
  action "Assinar o Pro" icon-end:arrow-right
}
```

</td></tr></table>

#### Prompt bar

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<BorderBeam size={80} duration={5} className="w-full max-w-md">
  <div className="flex items-center gap-3 rounded-2xl bg-surface-raised px-4 py-3">
    <Sparkles className="size-4 shrink-0 text-primary" aria-hidden="true" />
    <span className="flex-1 truncate text-sm text-fg-tertiary">
      Pergunte qualquer coisa ao Cronus…
    </span>
    <Button size="icon" variant="primary" aria-label="Enviar">
      <ArrowRight aria-hidden="true" className="size-4" />
    </Button>
  </div>
</BorderBeam>
```

</td><td>

```cronus
component PromptBar layout:stack style:border-beam+bar size:80 duration:5 width:md icon:sparkles icon-tone:primary {
  text "Pergunte qualquer coisa ao Cronus…"
  action "Enviar" size:icon icon:arrow-right
}
```

</td></tr></table>

#### Custom colours & reverse

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<BorderBeam
  colorFrom="var(--cronus-fg)"
  colorTo="transparent"
  size={90}
  duration={5}
  reverse
  className="w-full max-w-xs"
>
  <div className="flex flex-col gap-3 rounded-2xl bg-surface-raised p-6">
    <span className="grid size-9 place-items-center rounded-lg bg-surface-overlay text-fg">
      <ShieldCheck className="size-4" aria-hidden="true" />
    </span>
    <h3 className="font-display text-base font-semibold text-fg">Pagamentos protegidos</h3>
    <p className="text-sm text-fg-secondary">
      Antifraude e 3-D Secure em cada transação — o feixe reverso mantém o olhar na borda.
    </p>
  </div>
</BorderBeam>
```

</td><td>

```cronus
component Reverse layout:stack style:border-beam size:90 duration:5 reverse:true width:xs gap:3 icon:shield-check {
  title "Pagamentos protegidos"
  text "Antifraude e 3-D Secure em cada transação — o feixe reverso mantém o olhar na borda."
}
```

</td></tr></table>

### FlipCard (`/flip-card`)

#### Hover to flip

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<FlipCard aria-label="Plano Pro" className="h-72 w-full max-w-xs">
  <FlipCardFront className="justify-between p-6">
    <span className="grid size-11 place-items-center rounded-xl bg-surface-overlay text-fg">
      <Sparkles className="size-5" aria-hidden="true" />
    </span>
    <div>
      <h3 className="font-display text-lg font-semibold text-fg">Plano Pro</h3>
      <p className="mt-1 text-sm text-fg-secondary">
        Tudo o que você precisa para escalar a sua loja.
      </p>
    </div>
    <span className="text-xs font-medium text-fg-tertiary">Passe o mouse →</span>
  </FlipCardFront>
  <FlipCardBack className="justify-between p-6">
    <ul className="flex flex-col gap-2.5 text-sm text-fg-secondary">
      <li className="flex items-center gap-2">
        <Check className="size-4 shrink-0 text-primary" aria-hidden="true" />
        Repasses em D+2
      </li>
      <li className="flex items-center gap-2">
        <Check className="size-4 shrink-0 text-primary" aria-hidden="true" />
        Checkout sem marca
      </li>
      <li className="flex items-center gap-2">
        <Check className="size-4 shrink-0 text-primary" aria-hidden="true" />
        Suporte prioritário
      </li>
    </ul>
    <Button variant="primary" className="w-full">
      Assinar o Pro
      <ArrowRight aria-hidden="true" className="size-4" />
    </Button>
  </FlipCardBack>
</FlipCard>
```

</td><td>

```cronus
component PlanoPro layout:stack style:flip-card aria-label:"Plano Pro" width:xs height:72 icon:sparkles icon-size:lg {
  title "Plano Pro" size:lg
  text "Tudo o que você precisa para escalar a sua loja." mt:1
  meta "Passe o mouse →"
  slot "back"
  item "Repasses em D+2" icon:check
  item "Checkout sem marca" icon:check
  item "Suporte prioritário" icon:check
  action "Assinar o Pro" icon-end:arrow-right
}
```

</td></tr></table>

#### Click to flip

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<FlipCard
  trigger="click"
  aria-label="Ver depoimento de Ana Ribeiro"
  className="h-72 w-full max-w-xs"
>
  <FlipCardFront className="items-center justify-center gap-3 p-6 text-center">
    <span className="grid size-16 place-items-center rounded-full bg-surface-overlay text-lg font-semibold text-fg-secondary">
      AR
    </span>
    <div>
      <p className="font-display text-base font-semibold text-fg">Ana Ribeiro</p>
      <p className="text-sm text-fg-tertiary">Head of Design, Northwind</p>
    </div>
    <span className="text-xs text-fg-tertiary">Clique para ler</span>
  </FlipCardFront>
  <FlipCardBack className="items-center justify-center gap-4 p-6 text-center">
    <div className="flex gap-0.5 text-primary">
      {[1, 2, 3, 4, 5].map((n) => (
        <Star key={n} className="size-4 fill-current" aria-hidden="true" />
      ))}
    </div>
    <p className="text-sm leading-relaxed text-fg">
      “Shipped a polished, on-brand UI in a weekend. The theming alone paid for itself.”
    </p>
    <div className="flex items-center gap-3 text-fg-tertiary">
      <GithubGlyph className="size-4" aria-hidden="true" />
      <LinkedinGlyph className="size-4" aria-hidden="true" />
    </div>
  </FlipCardBack>
</FlipCard>
```

</td><td>

```cronus
component Depoimento layout:stack style:flip-card trigger:click aria-label:"Ver depoimento de Ana Ribeiro" width:xs height:72 {
  slot "front" layout:center avatar:"AR"
  title "Ana Ribeiro"
  text "Head of Design, Northwind" mt:0
  meta "Clique para ler"
  slot "back" layout:center rating:5 icons:"github,link-2"
  text "“Shipped a polished, on-brand UI in a weekend. The theming alone paid for itself.”" tone:fg
}
```

</td></tr></table>

#### Controlled

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
function OrderCard() {
  const [flipped, setFlipped] = useState(false);
  return (
    <div className="flex flex-col items-center gap-4">
      <FlipCard
        trigger="controlled"
        flipped={flipped}
        axis="vertical"
        aria-label="Detalhe do pedido"
        className="h-64 w-full max-w-xs"
      >
        <FlipCardFront className="justify-between p-6">
          <div className="flex items-center justify-between">
            <span className="text-xs font-medium uppercase tracking-wide text-fg-tertiary">
              Pedido #4821
            </span>
            <span className="rounded-full bg-success/15 px-2.5 py-0.5 text-xs font-medium text-success-strong">
              Pago
            </span>
          </div>
          <div>
            <p className="font-display text-3xl font-semibold text-fg">R$ 297,00</p>
            <p className="mt-1 text-sm text-fg-secondary">Curso de Copywriting</p>
          </div>
          <p className="text-xs text-fg-tertiary">Toque em “Ver detalhes”.</p>
        </FlipCardFront>
        <FlipCardBack className="justify-between p-6">
          <p className="text-xs font-medium uppercase tracking-wide text-fg-tertiary">Composição</p>
          <dl className="flex flex-col gap-2 text-sm">
            <div className="flex items-center justify-between">
              <dt className="text-fg-secondary">Subtotal</dt>
              <dd className="tabular-nums text-fg">R$ 320,00</dd>
            </div>
            <div className="flex items-center justify-between">
              <dt className="text-fg-secondary">Cupom BEMVINDO</dt>
              <dd className="tabular-nums text-success">− R$ 23,00</dd>
            </div>
            <div className="flex items-center justify-between border-t border-border pt-2 font-medium">
              <dt className="text-fg">Total</dt>
              <dd className="tabular-nums text-fg">R$ 297,00</dd>
            </div>
          </dl>
        </FlipCardBack>
      </FlipCard>
      <Button size="sm" variant="outline" onClick={() => setFlipped((value) => !value)}>
        <RotateCw aria-hidden="true" className="size-4" />
        {flipped ? "Ver resumo" : "Ver detalhes"}
      </Button>
    </div>
  );
}
```

</td><td>

```cronus
component Pedido layout:stack style:flip-card trigger:controlled axis:vertical aria-label:"Detalhe do pedido" width:xs height:64 {
  meta "Pedido #4821" eyebrow:true
  badge "Pago" tone:success
  value "R$ 297,00" size:3xl
  text "Curso de Copywriting" mt:1
  text "Toque em “Ver detalhes”." size:xs
  slot "back"
  meta "Composição" eyebrow:true
  item "Subtotal" value:"R$ 320,00"
  item "Cupom BEMVINDO" value:"− R$ 23,00" tone:success
  item "Total" value:"R$ 297,00" total:true
  action "Ver detalhes" swap:"Ver resumo" icon:rotate-cw
}
```

</td></tr></table>

### TiltCard (`/tilt-card`)

#### Glare & parallax

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<TiltCard glare parallax maxTilt={14} className="w-full max-w-xs">
  <div className="flex flex-col gap-3">
    <span className="grid size-11 place-items-center rounded-xl bg-surface-overlay text-fg">
      <Zap className="size-5" aria-hidden="true" />
    </span>
    <h3 className="font-display text-lg font-semibold text-fg">Repasses instantâneos</h3>
    <p className="text-sm text-fg-secondary">
      O saldo entra no mesmo instante em que a venda é aprovada — sem lote noturno, sem espera.
    </p>
  </div>
</TiltCard>
```

</td><td>

```cronus
component GlareParallax layout:stack style:tilt-card glare:true parallax:true width:xs icon:zap icon-size:lg {
  title "Repasses instantâneos" size:lg
  text "O saldo entra no mesmo instante em que a venda é aprovada — sem lote noturno, sem espera."
}
```

</td></tr></table>

#### Payment card

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<TiltCard
  glare
  parallax
  maxTilt={16}
  scale={1.05}
  className="w-full max-w-sm bg-fg text-fg-inverse"
>
  <div className="flex flex-col gap-6">
    <div className="flex items-start justify-between">
      <span className="font-display text-lg font-semibold">Cronus</span>
      <Wifi className="size-6 rotate-90 opacity-90" aria-hidden="true" />
    </div>
    <div className="h-9 w-12 rounded-md bg-fg-inverse/20 ring-1 ring-fg-inverse/15" aria-hidden="true" />
    <div className="flex flex-col gap-4">
      <p className="font-mono text-xl tracking-[0.25em]">4242 4242 4242 4242</p>
      <div className="flex items-center justify-between text-xs uppercase tracking-wide opacity-90">
        <span>Pedro Gontijo</span>
        <span className="tabular-nums">12/29</span>
      </div>
    </div>
  </div>
</TiltCard>
```

</td><td>

```cronus
component PaymentCard layout:stack style:tilt-card+payment glare:true parallax:true scale:1.05 width:sm icon-end:wifi {
  title "Cronus"
  value "4242 4242 4242 4242"
  meta "Pedro Gontijo"
  meta "12/29"
}
```

</td></tr></table>

#### Subtle

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<TiltCard maxTilt={6} scale={1.02} className="w-full max-w-xs">
  <div className="flex items-center gap-4">
    <span className="grid size-11 place-items-center rounded-full bg-surface-overlay text-fg-secondary">
      <User className="size-5" aria-hidden="true" />
    </span>
    <div className="min-w-0">
      <p className="truncate text-sm font-medium text-fg">Ana Ribeiro</p>
      <p className="truncate text-sm text-fg-tertiary">ana@cronus.app</p>
    </div>
    <ArrowRight className="ml-auto size-4 shrink-0 text-fg-tertiary" aria-hidden="true" />
  </div>
</TiltCard>
```

</td><td>

```cronus
component Subtle layout:stack style:tilt-card+row scale:1.02 width:xs icon:user icon-size:lg icon-shape:round icon-tone:secondary icon-end:arrow-right {
  title "Ana Ribeiro"
  text "ana@cronus.app"
}
```

</td></tr></table>

### Magnetic (`/magnetic`)

#### Magnetic call-to-action

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Magnetic className="p-10">
  <Button size="lg" className="rounded-full px-8 shadow-glow">
    Começar agora
    <ArrowRight className="size-4" aria-hidden="true" />
  </Button>
</Magnetic>
```

</td><td>

```cronus
component Cta layout:inline style:magnetic padding:10 {
  action "Começar agora" size:lg icon-end:arrow-right shape:pill glow:true px:8
}
```

</td></tr></table>

#### Icon row

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex items-center gap-1">
  {[
    { label: "GitHub", icon: GithubGlyph },
    { label: "LinkedIn", icon: LinkedinGlyph },
    { label: "Compartilhar", icon: Share2 },
  ].map(({ label, icon: Icon }) => (
    <Magnetic key={label} strength={0.25} radius={60} className="p-3">
      <Button variant="ghost" size="icon" aria-label={label} className="rounded-full">
        <Icon className="size-4" aria-hidden="true" />
      </Button>
    </Magnetic>
  ))}
</div>
```

</td><td>

```cronus
component IconRow layout:inline style:magnetic strength:0.25 padding:3 {
  action "GitHub" variant:ghost size:icon icon:github shape:pill
  action "LinkedIn" variant:ghost size:icon icon:link-2 shape:pill
  action "Compartilhar" variant:ghost size:icon icon:share-2 shape:pill
}
```

</td></tr></table>

#### Strength & radius

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="flex flex-wrap items-end justify-center gap-8">
  <div className="flex flex-col items-center gap-2">
    <Magnetic strength={0.15} radius={80} className="p-8">
      <Button variant="outline" className="rounded-full">Sutil</Button>
    </Magnetic>
    <span className="font-mono text-xs text-fg-tertiary tabular-nums">strength 0.15 · radius 80</span>
  </div>
  <div className="flex flex-col items-center gap-2">
    <Magnetic strength={0.6} radius={160} className="p-8">
      <Button variant="outline" className="rounded-full">Grudento</Button>
    </Magnetic>
    <span className="font-mono text-xs text-fg-tertiary tabular-nums">strength 0.6 · radius 160</span>
  </div>
</div>
```

</td><td>

```cronus
component FieldTuning layout:inline style:magnetic padding:8 {
  action "Sutil" variant:outline shape:pill strength:0.15 caption:"strength 0.15 · radius 80"
  action "Grudento" variant:outline shape:pill strength:0.6 caption:"strength 0.6 · radius 160"
}
```

</td></tr></table>

### Orbit (`/orbit`)

#### Integration constellation

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Orbit aria-label="Tools orbiting the product core" className="size-80">
  <span className="grid size-14 place-items-center rounded-2xl bg-surface-overlay text-fg">
    <Zap className="size-6" />
  </span>
  <OrbitRing radius={72} duration={22}>
    {innerTools.map(({ label, icon: Icon }) => (
      <OrbitItem key={label}>
        <span
          role="img"
          aria-label={label}
          className="grid size-10 place-items-center rounded-full border border-border bg-surface-raised text-fg-secondary shadow-sm"
        >
          <Icon className="size-4" />
        </span>
      </OrbitItem>
    ))}
  </OrbitRing>
  <OrbitRing radius={128} duration={36} reverse startAngle={36}>
    {outerTools.map(({ label, icon: Icon }) => ( /* …same chip… */ ))}
  </OrbitRing>
</Orbit>
```

</td><td>

```cronus
component Constellation layout:inline style:orbit aria-label:"Tools orbiting the product core" size:80 icon:zap {
  slot "ring" radius:72 duration:22
  item "Search" icon:search
  item "Alerts" icon:bell
  item "Settings" icon:settings
  slot "ring" radius:128 duration:36 reverse:true start:36
  item "GitHub" icon:github
  item "Security" icon:shield-check
  item "Chat" icon:message-square-plus
  item "Uptime" icon:wifi
  item "Favorites" icon:star
}
```

</td></tr></table>

#### Team halo

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Orbit aria-label="Teammates on call" className="size-64">
  <div className="flex flex-col items-center">
    <span className="font-display text-3xl font-semibold tabular-nums text-fg">04</span>
    <span className="text-xs text-fg-tertiary">on call</span>
  </div>
  <OrbitRing radius={96} duration={30} guide={false} startAngle={45}>
    {team.map((member) => (
      <OrbitItem key={member.name}>
        <span
          title={member.name}
          className="grid size-9 place-items-center rounded-full border border-border bg-surface-overlay text-xs font-medium text-fg-secondary shadow-xs"
        >
          {member.initials}
        </span>
      </OrbitItem>
    ))}
  </OrbitRing>
</Orbit>
```

</td><td>

```cronus
component TeamHalo layout:inline style:orbit aria-label:"Teammates on call" size:64 {
  value "04" meta:"on call"
  slot "ring" radius:96 duration:30 guide:false start:45
  item "Ana Ribeiro" initials:AR
  item "Marcus Lee" initials:ML
  item "Priya Nair" initials:PN
  item "Tom Costa" initials:TC
}
```

</td></tr></table>

### Terminal (`/terminal`)

#### Install session

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Terminal
  title="zsh"
  loop
  lines={[
    { type: "input", text: "npx cronus-ui add terminal" },
    { type: "output", text: "✔ 1 component installed" },
    { type: "output", text: "  src/components/ui/terminal.tsx" },
    { type: "input", text: "bun run dev" },
    { type: "output", text: "ready in 312 ms" },
  ]}
/>
```

</td><td>

```cronus
component InstallSession layout:stack style:terminal motion:respect {
  label "zsh"
  item "npx cronus-ui add terminal"
  text "✔ 1 component installed"
  text "  src/components/ui/terminal.tsx"
  item "bun run dev"
  text "ready in 312 ms"
}
```

</td></tr></table>

#### Static, chrome-less log

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Terminal
  chrome={false}
  motionPreference="never"
  lines={[
    { type: "input", text: "cronus deploy --prod" },
    { type: "output", text: "Build completed in 8.2s" },
    { type: "output", text: "Deployed to https://app.cronus.com" },
  ]}
/>
```

</td><td>

```cronus
component StaticLog layout:stack style:terminal chrome:false motion:never {
  item "cronus deploy --prod"
  text "Build completed in 8.2s"
  text "Deployed to https://app.cronus.com"
}
```

</td></tr></table>

### Ripple (`/ripple`)

#### Pulse

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Ripple className="grid min-h-56 place-items-center rounded-2xl border border-border bg-surface-raised">
  <p className="font-display text-2xl text-fg">Now live</p>
</Ripple>
```

</td><td>

```cronus
component Pulse layout:stack style:ripple surface:raised size:2xl { label "Now live" }
```

</td></tr></table>

### Meteors (`/meteors`)

#### Shooting stars

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Meteors className="grid min-h-56 place-items-center rounded-2xl border border-border bg-surface-raised">
  <p className="font-display text-2xl text-fg">Launch window</p>
</Meteors>
```

</td><td>

```cronus
component Field layout:stack style:meteors surface:raised size:2xl { label "Launch window" }
```

</td></tr></table>

### DotPattern (`/dot-pattern`)

#### Dotted field

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<DotPattern className="grid min-h-56 place-items-center rounded-2xl border border-border bg-surface-raised">
  <p className="font-display text-2xl text-fg">Quiet texture</p>
</DotPattern>
```

</td><td>

```cronus
component QuietTexture layout:stack style:dot-pattern card:true heading:2xl { label "Quiet texture" }
```

</td></tr></table>

### GridPattern (`/grid-pattern`)

#### Grid field

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<GridPattern className="grid min-h-56 place-items-center rounded-2xl border border-border bg-surface-raised">
  <p className="font-display text-2xl text-fg">Blueprint</p>
</GridPattern>
```

</td><td>

```cronus
component Blueprint layout:stack style:grid-pattern card:true heading:2xl { label "Blueprint" }
```

</td></tr></table>

### RetroGrid (`/retro-grid`)

#### Perspective floor

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<RetroGrid className="grid min-h-64 place-items-center rounded-2xl border border-border bg-surface-raised">
  <p className="font-display text-2xl text-fg">Horizon</p>
</RetroGrid>
```

</td><td>

```cronus
component Horizon layout:stack style:retro-grid card:true heading:2xl { label "Horizon" }
```

</td></tr></table>

### Noise (`/noise`)

#### Film grain

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Noise className="grid min-h-56 place-items-center rounded-2xl border border-border bg-surface-raised">
  <p className="font-display text-2xl text-fg">Print</p>
</Noise>
```

</td><td>

```cronus
component Print layout:stack style:noise card:true heading:2xl { label "Print" }
```

</td></tr></table>

### LightRays (`/light-rays`)

#### Light shafts

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<LightRays className="grid min-h-56 place-items-center rounded-2xl border border-border bg-surface-raised">
  <p className="font-display text-2xl text-fg">Dawn</p>
</LightRays>
```

</td><td>

```cronus
component Dawn layout:stack style:light-rays card:true heading:2xl { label "Dawn" }
```

</td></tr></table>

### ProgressiveBlur (`/progressive-blur`)

#### Edge fade

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<div className="relative h-48 w-full overflow-hidden rounded-2xl border border-border bg-surface-raised">
  <div className="h-full overflow-y-auto p-4 pb-16">
    <p className="text-sm text-fg-secondary">
      Scroll under the blur. The band is decorative and sits on the edge of the region.
    </p>
    <p className="mt-4 text-sm text-fg-secondary">More copy so the panel actually scrolls.</p>
    <p className="mt-4 text-sm text-fg-secondary">Keep going — the fade holds the last lines.</p>
    <p className="mt-4 text-sm text-fg-secondary">Last line of the stack.</p>
  </div>
  <ProgressiveBlur side="bottom" />
</div>
```

</td><td>

```cronus
component EdgeFade layout:stack style:progressive-blur card:true side:bottom {
  text "Scroll under the blur. The band is decorative and sits on the edge of the region."
  text "More copy so the panel actually scrolls."
  text "Keep going — the fade holds the last lines."
  text "Last line of the stack."
}
```

</td></tr></table>

### FlickeringGrid (`/flickering-grid`)

#### Signal grid

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<FlickeringGrid className="grid min-h-56 place-items-center rounded-2xl border border-border bg-surface-raised">
  <p className="font-display text-2xl text-fg">Signal</p>
</FlickeringGrid>
```

</td><td>

```cronus
component Signal layout:stack style:flickering-grid card:true heading:2xl { label "Signal" }
```

</td></tr></table>

### StarBorder (`/star-border`)

#### Twinkle border

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<StarBorder className="rounded-2xl border border-border bg-surface-raised p-8">
  <p className="font-display text-xl text-fg">Featured</p>
</StarBorder>
```

</td><td>

```cronus
component Featured layout:stack style:star-border card:true heading:xl { label "Featured" }
```

</td></tr></table>

### ShinyText (`/shiny-text`)

#### Sheen

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<ShinyText className="font-display text-4xl">Ship the surface</ShinyText>
```

</td><td>

```cronus
component Sheen layout:inline style:shiny-text heading:4xl { label "Ship the surface" }
```

</td></tr></table>

### Highlighter (`/highlighter`)

#### Marker

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<p className="font-display text-3xl text-fg">
  Build the <Highlighter>product surface</Highlighter> first.
</p>
```

</td><td>

```cronus
component Marker layout:inline style:highlighter prefix:"Build the " suffix:" first." heading:3xl { label "product surface" }
```

</td></tr></table>

### SpinningText (`/spinning-text`)

#### Orbit

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<SpinningText radius={56}>cronus ui · product · </SpinningText>
```

</td><td>

```cronus
component Orbit layout:inline style:spinning-text radius:56 { label "cronus ui · product · " }
```

</td></tr></table>

### SparklesText (`/sparkles-text`)

#### Sparkles

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<SparklesText className="font-display text-4xl text-fg">Launch</SparklesText>
```

</td><td>

```cronus
component SparklesLaunch layout:inline style:sparkles-text heading:4xl { label "Launch" }
```

</td></tr></table>

### TypingText (`/typing-text`)

#### Typewriter

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<TypingText
  className="font-display text-3xl text-fg"
  text={["Design systems.", "Product surfaces.", "Copy you can upgrade."]}
/>
```

</td><td>

```cronus
component Typewriter layout:inline style:typing-text heading:3xl {
  text "Design systems."
  text "Product surfaces."
  text "Copy you can upgrade."
}
```

</td></tr></table>

### WordRotate (`/word-rotate`)

#### Word cycle

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<p className="font-display text-3xl text-fg">
  Ship <WordRotate words={["faster", "calmer", "on-brand"]} />.
</p>
```

</td><td>

```cronus
component WordCycle layout:inline style:word-rotate prefix:"Ship " suffix:"." heading:3xl {
  item "faster"
  item "calmer"
  item "on-brand"
}
```

</td></tr></table>

### ScrambleText (`/scramble-text`)

#### Decrypt

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<ScrambleText className="font-display text-3xl text-fg">cronus-ui</ScrambleText>
```

</td><td>

```cronus
component Decrypt layout:inline style:scramble-text heading:3xl { label "cronus-ui" }
```

</td></tr></table>

### GlareHover (`/glare-hover`)

#### Pointer glare

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<GlareHover className="rounded-2xl border border-border bg-surface-raised p-8">
  <p className="font-display text-xl text-fg">Hover the surface</p>
</GlareHover>
```

</td><td>

```cronus
component HoverTheSurface layout:stack style:glare-hover card:true heading:xl { label "Hover the surface" }
```

</td></tr></table>

### ClickSpark (`/click-spark`)

#### Click burst

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<ClickSpark className="grid min-h-40 place-items-center rounded-2xl border border-border bg-surface-raised">
  <Button>Click me</Button>
</ClickSpark>
```

</td><td>

```cronus
component ClickMe layout:stack style:click-spark card:true { action "Click me" }
```

</td></tr></table>

### AnimatedList (`/animated-list`)

#### Staggered list

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<AnimatedList className="w-full max-w-sm">
  <div className="rounded-xl border border-border bg-surface-raised px-4 py-3 text-sm text-fg">
    Deploy finished
  </div>
  <div className="rounded-xl border border-border bg-surface-raised px-4 py-3 text-sm text-fg">
    Invite accepted
  </div>
  <div className="rounded-xl border border-border bg-surface-raised px-4 py-3 text-sm text-fg">
    Invoice paid
  </div>
</AnimatedList>
```

</td><td>

```cronus
component StaggeredList layout:stack style:animated-list card:true {
  item "Deploy finished"
  item "Invite accepted"
  item "Invoice paid"
}
```

</td></tr></table>

### CardStack (`/card-stack`)

#### Fanned stack

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<CardStack
  items={[
    { id: "one", content: <p className="font-display text-lg">Northwind</p> },
    { id: "two", content: <p className="font-display text-lg">Contoso</p> },
    { id: "three", content: <p className="font-display text-lg">Adventure Works</p> },
  ]}
/>
```

</td><td>

```cronus
component FannedStack layout:stack style:card-stack heading:lg {
  item "Northwind"
  item "Contoso"
  item "Adventure Works"
}
```

</td></tr></table>

### PillNav (`/pill-nav`)

#### Sliding pill

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<PillNav
  aria-label="Product sections"
  items={[
    { value: "overview", label: "Overview" },
    { value: "pricing", label: "Pricing" },
    { value: "docs", label: "Docs" },
  ]}
/>
```

</td><td>

```cronus
component SlidingPill layout:inline style:pill-nav aria-label:"Product sections" {
  item "Overview"
  item "Pricing"
  item "Docs"
}
```

</td></tr></table>

### ExpandableTabs (`/expandable-tabs`)

#### Expanding tabs

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<ExpandableTabs
  items={[
    { value: "home", label: "Home", icon: <Home /> },
    { value: "search", label: "Search", icon: <Search /> },
    { value: "settings", label: "Settings", icon: <Settings /> },
  ]}
/>
```

</td><td>

```cronus
component ExpandingTabs layout:inline style:expandable-tabs {
  item "Home" icon:home
  item "Search" icon:search
  item "Settings" icon:settings
}
```

</td></tr></table>

### ExploreNav (`/explore-nav`)

#### Product family

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<ExploreNav
  title="iPhone 17 Pro"
  products={products}
  links={links}
  buyHref="#buy"
/>
```

</td><td>

```cronus
component IphoneFamily layout:stack style:explore-nav buy:"#buy" {
  title "iPhone 17 Pro"
  item "iPhone 17 Pro" image:"https://skiper-ui.com/images/oct25Coll/iphone17/1.png" price:"From $1099" price-note:"or $45.79/mo. for 24 mo."
  item "iPhone 17" image:"https://skiper-ui.com/images/oct25Coll/iphone17/2.png" badge:"New"
  item "iPhone 17 Air" image:"https://skiper-ui.com/images/oct25Coll/iphone17/3.png" badge:"New"
  item "iPhone 16 Pro" image:"https://skiper-ui.com/images/oct25Coll/iphone17/4.png"
  item "iPhone 16" image:"https://skiper-ui.com/images/oct25Coll/iphone17/5.png"
  item "iPhone 16 E" image:"https://skiper-ui.com/images/oct25Coll/iphone17/6.png"
  item "Compare" image:"https://skiper-ui.com/images/oct25Coll/iphone17/7.png"
  item "Accessories" image:"https://skiper-ui.com/images/oct25Coll/iphone17/8.png"
  item "iOS" image:"https://skiper-ui.com/images/oct25Coll/iphone17/9.png"
  link "Highlights"
  link "Performance"
  link "Design"
  link "Cameras"
  link "Tech specs"
}
```

</td></tr></table>

### BouncyAccordion (`/bouncy-accordion`)

#### Bouncy stack

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<BouncyAccordion
  defaultValue="schedule"
  items={[
    { id: "type", title: "Type Shit", description: "Fast, accurate typing.", icon: <BookOpen /> },
    { id: "schedule", title: "Schedule", description: "Plan tasks with timelines.", icon: <Calendar /> },
  ]}
/>
```

</td><td>

```cronus
component BouncyStack layout:stack style:bouncy-accordion value:"Schedule" {
  item "Type Shit" icon:book-open description:"Fast, accurate typing."
  item "Schedule" icon:calendar description:"Plan tasks with timelines."
}
```

</td></tr></table>

### TokenSwap (`/token-swap`)

#### Aave swap

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<TokenSwap />
```

</td><td>

```cronus
component AaveSwap layout:stack style:token-swap { label "Swap" }
```

</td></tr></table>

### ReceiveButton (`/receive-button`)

#### Family receive

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<ReceiveButton />
```

</td><td>

```cronus
component FamilyReceive layout:stack style:receive-button { label "Receive" }
```

</td></tr></table>

### FamilyWallet (`/family-wallet`)

#### Sign in drawer

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<FamilyWallet />
```

</td><td>

```cronus
component FamilySignIn layout:stack style:family-wallet { label "Sign In" }
```

</td></tr></table>

### DynamicIsland (`/dynamic-island`)

#### Live activity

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<DynamicIsland
  views={[
    { id: "idle", label: "Idle", content: <span className="text-sm">Cronus</span> },
    {
      id: "now",
      label: "Now playing",
      content: <span className="text-sm">Shipping the surface</span>,
    },
  ]}
/>
```

</td><td>

```cronus
component LiveActivity layout:inline style:dynamic-island {
  item "Cronus" aria-label:"Idle"
  item "Shipping the surface" aria-label:"Now playing"
}
```

</td></tr></table>

### Confetti (`/confetti`)

#### Burst

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Confetti className="grid min-h-40 place-items-center rounded-2xl border border-border bg-surface-raised">
  <Button>Celebrate</Button>
</Confetti>
```

</td><td>

```cronus
component Celebrate layout:stack style:confetti card:true { action "Celebrate" }
```

</td></tr></table>

### Particles (`/particles`)

#### Drifting field

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Particles className="grid min-h-56 place-items-center rounded-2xl border border-border bg-surface-raised">
  <p className="font-display text-2xl text-fg">Atmosphere</p>
</Particles>
```

</td><td>

```cronus
component Atmosphere layout:stack style:particles card:true heading:2xl { label "Atmosphere" }
```

</td></tr></table>

## Chat & AI

### Actions (`/actions`)

#### Message actions

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Actions>
  <Action tooltip="Copy" label="Copy">
    <Copy />
  </Action>
  <Action tooltip="Retry" label="Retry">
    <RefreshCw />
  </Action>
  <Action tooltip="Share" label="Share">
    <Share />
  </Action>
</Actions>
```

</td><td>

```cronus
component MessageActions layout:inline style:actions {
  label "Message actions"
  action "Copy" icon:copy
  action "Retry" icon:refresh-cw
  action "Share" icon:share
}
```

</td></tr></table>

### Artifact (`/artifact`)

#### Generated artifact

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Artifact className="w-full max-w-lg">
  <ArtifactHeader>
    <ArtifactTitle>landing.tsx</ArtifactTitle>
  </ArtifactHeader>
  <ArtifactContent>
    <pre className="font-mono text-xs text-fg-secondary">export function Hero() {"{"}
  return &lt;h1&gt;Intelligence at scale.&lt;/h1&gt;
{"}"}</pre>
  </ArtifactContent>
</Artifact>
```

</td><td>

```cronus
component GeneratedArtifact layout:stack style:artifact code:true {
  title "landing.tsx"
  text "export function Hero() {"
  text "  return <h1>Intelligence at scale.</h1>"
  text "}"
}
```

</td></tr></table>

### Branch (`/branch`)

#### Alternative generations

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Branch>
  <BranchMessages>
    <p>First draft of the answer.</p>
    <p>A tighter rewrite of the same answer.</p>
  </BranchMessages>
  <BranchSelector from="assistant">
    <BranchPrevious />
    <BranchPage />
    <BranchNext />
  </BranchSelector>
</Branch>
```

</td><td>

```cronus
component AlternativeGenerations layout:stack style:branch from:assistant {
  label "Alternative generations"
  text "First draft of the answer."
  text "A tighter rewrite of the same answer."
}
```

</td></tr></table>

### ChainOfThought (`/chain-of-thought`)

#### Reasoning trail

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<ChainOfThought defaultOpen>
  <ChainOfThoughtHeader />
  <ChainOfThoughtContent>
    <ChainOfThoughtStep label="Parse the question" />
    <ChainOfThoughtStep label="Retrieve the docs" />
    <ChainOfThoughtStep label="Draft the answer" />
  </ChainOfThoughtContent>
</ChainOfThought>
```

</td><td>

```cronus
component ReasoningTrail layout:stack style:chain-of-thought open:true {
  label "Chain of Thought"
  item "Parse the question"
  item "Retrieve the docs"
  item "Draft the answer"
}
```

</td></tr></table>

### AiCodeBlock (`/ai-code-block`)

#### Tool output

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<AiCodeBlock
  defaultValue="main.ts"
  data={[{ filename: "main.ts", language: "ts", code: "export const n = 1\\n" }]}
>
  <AiCodeBlockHeader>
    <span className="font-mono text-xs">main.ts</span>
    <AiCodeBlockCopyButton />
  </AiCodeBlockHeader>
  <AiCodeBlockBody>
    {(item) => (
      <AiCodeBlockItem key={item.filename} value={item.filename}>
        <AiCodeBlockContent language={item.language}>{item.code}</AiCodeBlockContent>
      </AiCodeBlockItem>
    )}
  </AiCodeBlockBody>
</AiCodeBlock>
```

</td><td>

```cronus
component ToolOutput layout:stack style:ai-code-block value:"main.ts" {
  item "main.ts" language:ts
  text "export const n = 1"
}
```

</td></tr></table>

### Context (`/context`)

#### Token usage

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Context
  usedTokens={18432}
  maxTokens={128000}
  usage={{ inputTokens: 12000, outputTokens: 4432, reasoningTokens: 800, cachedInputTokens: 2048 }}
>
  <ContextTrigger />
  <ContextContent>
    <ContextContentHeader />
    <ContextContentBody>
      <ContextInputUsage />
      <ContextOutputUsage />
      <ContextReasoningUsage />
      <ContextCacheUsage />
    </ContextContentBody>
    <ContextContentFooter />
  </ContextContent>
</Context>
```

</td><td>

```cronus
component TokenUsage layout:inline style:context used:18432 max:128000 input:12000 output:4432 reasoning:800 cached:2048 { label "Model context usage" }
```

</td></tr></table>

### Conversation (`/conversation`)

#### Empty state

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Conversation className="h-64 rounded-xl border border-border">
  <ConversationContent>
    <ConversationEmptyState />
  </ConversationContent>
</Conversation>
```

</td><td>

```cronus
component EmptyLog layout:stack style:conversation bordered:true { label "Conversation" }
```

</td></tr></table>

### GeneratedImage (`/ai-image`)

#### Generated image

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<GeneratedImage
  alt="A 1×1 placeholder the model returned"
  base64="iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg=="
  mediaType="image/png"
  className="size-24"
/>
```

</td><td>

```cronus
component GeneratedImage layout:inline style:ai-image media-type:image/png base64:"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==" width:96 height:96 { label "A 1×1 placeholder the model returned" }
```

</td></tr></table>

### InlineCitation (`/inline-citation`)

#### Cited claim

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<p className="text-sm text-fg">
  Cronus is a product UI system
  <InlineCitation>
    <InlineCitationText>*</InlineCitationText>
    <InlineCitationCard>
      <InlineCitationCardTrigger sources={["https://aicronus.com"]} />
      <InlineCitationCardBody>
        <InlineCitationSource title="Cronus UI" url="https://aicronus.com" />
      </InlineCitationCardBody>
    </InlineCitationCard>
  </InlineCitation>
  .
</p>
```

</td><td>

```cronus
component CitedClaim layout:inline style:inline-citation {
  label "*"
  link "Cronus UI" -> "https://aicronus.com"
}
```

</td></tr></table>

### Loader (`/loader`)

#### Generating

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Loader />
```

</td><td>

```cronus
component Generating layout:inline style:loader { label "Loading" }
```

</td></tr></table>

### Message (`/message`)

#### User and assistant

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<>
  <Message from="user">
    <MessageContent>How do I theme a surface?</MessageContent>
  </Message>
  <Message from="assistant">
    <MessageAvatar src="https://github.com/pedrogbraz.png" name="Cronus" />
    <MessageContent>Use semantic tokens. Never a palette scale.</MessageContent>
  </Message>
</>
```

</td><td>

```cronus
component UserTurn layout:stack style:message from:user { text "How do I theme a surface?" }
component AssistantTurn layout:stack style:message from:assistant avatar:"https://github.com/pedrogbraz.png" name:"Cronus" { text "Use semantic tokens. Never a palette scale." }
```

</td></tr></table>

### OpenIn (`/open-in-chat`)

#### Open elsewhere

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<OpenIn query="Explain Cronus tokens">
  <OpenInTrigger />
  <OpenInContent>
    <OpenInLabel>Open in</OpenInLabel>
    <OpenInSeparator />
    <OpenInChatGPT />
    <OpenInClaude />
    <OpenInCursor />
  </OpenInContent>
</OpenIn>
```

</td><td>

```cronus
component OpenElsewhere layout:inline style:open-in-chat query:"Explain Cronus tokens" heading:"Open in" {
  label "Open in chat"
  item "chatgpt"
  item "claude"
  item "cursor"
}
```

</td></tr></table>

### Plan (`/plan`)

#### Plan card

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Plan defaultOpen>
  <PlanHeader>
    <PlanTitle>Ship the chat surface</PlanTitle>
    <PlanTrigger />
  </PlanHeader>
  <PlanContent>
    <p className="text-sm text-fg-secondary">Wire Conversation, Message, and PromptInput.</p>
  </PlanContent>
</Plan>
```

</td><td>

```cronus
component PlanCard layout:stack style:plan open:true {
  title "Ship the chat surface"
  text "Wire Conversation, Message, and PromptInput."
}
```

</td></tr></table>

### PromptInput (`/prompt-input`)

#### Composer

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<PromptInput onSubmit={(message) => console.log(message)}>
  <PromptInputBody>
    <PromptInputTextarea placeholder="Ask anything…" />
  </PromptInputBody>
  <PromptInputFooter>
    <PromptInputTools />
    <PromptInputSubmit />
  </PromptInputFooter>
</PromptInput>
```

</td><td>

```cronus
component Composer layout:stack style:prompt-input placeholder:"Ask anything…" { label "Ask anything…" }
```

</td></tr></table>

### Queue (`/queue`)

#### Pending work

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Queue>
  <QueueList>
    <QueueItem>
      <QueueItemIndicator />
      <QueueItemContent>Summarise the last three PRs</QueueItemContent>
    </QueueItem>
    <QueueItem>
      <QueueItemIndicator />
      <QueueItemContent>Draft the changelog</QueueItemContent>
    </QueueItem>
  </QueueList>
</Queue>
```

</td><td>

```cronus
component PendingWork layout:stack style:queue {
  label "Pending work"
  item "Summarise the last three PRs"
  item "Draft the changelog"
}
```

</td></tr></table>

### Reasoning (`/reasoning`)

#### Thinking

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Reasoning defaultOpen duration={4}>
  <ReasoningTrigger />
  <ReasoningContent>
    The user wants a product UI system, not a bag of parts.
  </ReasoningContent>
</Reasoning>
```

</td><td>

```cronus
component Thinking layout:stack style:reasoning defaultOpen:true duration:4 { text "The user wants a product UI system, not a bag of parts." }
```

</td></tr></table>

### Response (`/response`)

#### Assistant text

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Response>
  Cronus tokens re-theme live. Compose the app; do not fork the kit.
</Response>
```

</td><td>

```cronus
component AssistantText layout:stack style:response { text "Cronus tokens re-theme live. Compose the app; do not fork the kit." }
```

</td></tr></table>

### Sources (`/sources`)

#### Used sources

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Sources>
  <SourcesTrigger count={2} />
  <SourcesContent>
    <Source href="https://aicronus.com" title="Cronus UI" />
    <Source href="https://aicronus.com/docs/design" title="Design" />
  </SourcesContent>
</Sources>
```

</td><td>

```cronus
component UsedSources layout:stack style:sources count:2 {
  link "Cronus UI" -> "https://aicronus.com"
  link "Design" -> "https://aicronus.com/docs/design"
}
```

</td></tr></table>

### Suggestions (`/suggestion`)

#### Prompt chips

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Suggestions>
  <Suggestion suggestion="Explain tokens" />
  <Suggestion suggestion="Show a login" />
  <Suggestion suggestion="Compose a SaaS" />
</Suggestions>
```

</td><td>

```cronus
component PromptChips layout:stack style:suggestion {
  item "Explain tokens"
  item "Show a login"
  item "Compose a SaaS"
}
```

</td></tr></table>

### Task (`/task`)

#### Agent checklist

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Task defaultOpen>
  <TaskTrigger title="Preparing the PR" />
  <TaskContent>
    <TaskItem>Read the contract</TaskItem>
    <TaskItem>Port the component</TaskItem>
    <TaskItem>Verify in the browser</TaskItem>
  </TaskContent>
</Task>
```

</td><td>

```cronus
component AgentChecklist layout:stack style:task {
  title "Preparing the PR"
  item "Read the contract"
  item "Port the component"
  item "Verify in the browser"
}
```

</td></tr></table>

### TextShimmer (`/text-shimmer`)

#### Thinking placeholder

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<TextShimmer>Thinking…</TextShimmer>
```

</td><td>

```cronus
component ThinkingPlaceholder layout:inline style:text-shimmer { label "Thinking…" }
```

</td></tr></table>

### Tool (`/tool`)

#### Completed tool

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<Tool defaultOpen>
  <ToolHeader type="tool-search" state="output-available" title="search" />
  <ToolContent>
    <ToolInput input={{ query: "Cronus tokens" }} />
    <ToolOutput output="Semantic tokens live in @cronus-ui/tokens." />
  </ToolContent>
</Tool>
```

</td><td>

```cronus
component CompletedTool layout:stack style:tool defaultOpen:true type:tool-search state:output-available title:search output:"Semantic tokens live in @cronus-ui/tokens." {
  field "query" value:"Cronus tokens"
}
```

</td></tr></table>

### WebPreview (`/web-preview`)

#### Sandboxed preview

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
<WebPreview defaultUrl="https://aicronus.com">
  <WebPreviewNavigation>
    <WebPreviewUrl />
  </WebPreviewNavigation>
  <WebPreviewBody className="h-56" />
</WebPreview>
```

</td><td>

```cronus
component SandboxedPreview layout:stack style:web-preview url:"https://aicronus.com" { label "Preview" }
```

</td></tr></table>

## Preloaders

### WordsPreloader (`/words-preloader`)

#### Product words

<table><tr><th>React</th><th>.cronus</th></tr><tr><td>

```tsx
const [show, setShow] = useState(true);

return (
  <main className="relative">
    <AnimatePresence>
      {show ? (
        <WordsPreloader
          end={<BrandMark className="h-16 w-32 text-fg" />}
          onComplete={() => setShow(false)}
        />
      ) : null}
    </AnimatePresence>
    <Page />
  </main>
);
```

</td><td>

```cronus
component ProductWords layout:stack style:words-preloader+contained end:"Cronus" {
  label "Loading"
  item "The innovation of interfaces."
  item "One system. The whole product follows."
}
```

</td></tr></table>
