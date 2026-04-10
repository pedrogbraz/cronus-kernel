# .cronus Section Types Quick Reference

## Data Sections (REQUIRE bind)

### table — Data table with search, sort, pagination
```cronus
section table {
  bind Product { query all order name asc }
  columns "name, price, stock, category"
  search true
}
```

### kpi — Single metric card
```cronus
section kpi {
  bind Order { aggregate count }
  item "Total Orders" value:bind icon:shopping_cart
}
section kpi {
  bind Order { aggregate sum field:total }
  item "Revenue" value:bind icon:attach_money
}
```

### chart — Bar/line/pie chart from aggregated data
```cronus
section chart {
  bind Order { aggregate sum field:total group_by:created_at interval:month }
  chart_type bar
}
```

### kanban — Kanban board grouped by enum field
```cronus
section kanban {
  bind Task { query all }
  columns "todo, doing, done"
}
```

### timeline — Chronological event list
```cronus
section timeline {
  bind Event { query all order created_at desc limit 20 }
}
```

## Form Sections

### form — Data entry form (REQUIRES on submit)
```cronus
section form {
  bind Product
  on submit {
    create Product
    toast "Created" success
    navigate "/products"
  }
}
```

### modal — Popup form/content
```cronus
section modal {
  title "Confirm Delete"
  on submit confirm:"Are you sure?" {
    delete Product route.id
    toast "Deleted" success
    navigate "/products"
  }
}
```

## Navigation Sections

### tabs — Tab navigation
```cronus
section tabs {
  item "Overview" active:true
  item "Settings"
  item "Billing"
}
```

### breadcrumb — Breadcrumb trail
```cronus
section breadcrumb {
  item "Home" href:"/"
  item "Products" href:"/products"
  item "Edit"
}
```

## Marketing Sections (for landing pages)

### hero — Full-width banner with CTA
```cronus
section hero {
  title "Build Faster"
  subtitle "Ship in minutes, not months"
  item "Get Started" action:"/signup" style:primary
  item "Learn More" action:"/docs" style:secondary
}
```

### features — Feature grid (2-4 columns)
```cronus
section features cols:3 {
  item "Fast" icon:bolt {
    "Compiled to native Rust"
  }
  item "Secure" icon:shield {
    "Built-in auth and audit"
  }
  item "Simple" icon:code {
    "50 lines = full app"
  }
}
```

### pricing — Pricing plans with CTA
```cronus
section pricing {
  item "Starter" price:"$9/mo" {
    "5 projects"
    "1GB storage"
    action "Choose Plan"
  }
  item "Pro" price:"$29/mo" featured:true {
    "Unlimited projects"
    "10GB storage"
    action "Choose Plan"
  }
}
```

### cta — Call to action banner
```cronus
section cta {
  title "Ready to start?"
  subtitle "Get up and running in 5 minutes"
  item "Sign Up Free" action:"/signup"
}
```

### faq — Accordion Q&A
```cronus
section faq {
  item "What is CRONUS?" {
    "A declarative full-stack language"
  }
  item "How much does it cost?" {
    "Free and open source"
  }
}
```

### testimonial — Customer quotes
```cronus
section testimonial {
  item "Jane Doe" meta:"CEO, Acme" {
    "Best framework I've ever used"
  }
}
```

### footer — Page footer with links
```cronus
section footer {
  copyright "2026 Company"
  item "Privacy" href:"/privacy"
  item "Terms" href:"/terms"
  item "Contact" href:"/contact"
}
```

## Feedback Sections

### alert — Status message
```cronus
section alert style:warning {
  title "Maintenance scheduled for tonight"
}
```

### empty — Empty state placeholder
```cronus
section empty {
  title "No products yet"
  subtitle "Create your first product"
  item "Add Product" action:"/products/new" icon:add
}
```

## Layout Sections

### card — Generic content card
```cronus
section card {
  bind Product { query one where id eq:route.id }
}
```

### page-header — Page title with actions
```cronus
section page-header {
  title "Products"
  subtitle "Manage your catalog"
  item "New Product" action:"/products/new" icon:add
}
```
