# Common .cronus Mistakes AI Makes

## WRONG: Using React/JSX syntax
```
// WRONG - this is React, not .cronus
export default function Page() {
  return <div><h1>Hello</h1></div>
}
```
```cronus
// CORRECT
page "/" {
  section hero {
    title "Hello"
  }
}
```

## WRONG: Adding id/timestamp fields
```cronus
// WRONG - CRONUS auto-generates these
entity User {
  id         string
  name       string!
  created_at date
  updated_at date
}
```
```cronus
// CORRECT - only your fields
entity User {
  name string!
}
```

## WRONG: Hardcoding data in sections
```cronus
// WRONG - lint error C001
section kpi {
  item "Revenue" value:"$45,000"
}
```
```cronus
// CORRECT - bind to database
section kpi {
  bind Order { aggregate sum field:total }
  item "Revenue" value:bind icon:attach_money
}
```

## WRONG: `required` keyword instead of `!`
```cronus
// WRONG - verbose
entity Product {
  name string required unique
}
```
```cronus
// CORRECT - use ! shorthand
entity Product {
  name string! unique
}
```

## WRONG: `bind entity:X` instead of `bind X`
```cronus
// WRONG
section table {
  bind entity:Product { query all }
}
```
```cronus
// CORRECT
section table {
  bind Product { query all }
}
```

## WRONG: Form without on submit
```cronus
// WRONG - lint error C012
section form {
  bind Product
}
```
```cronus
// CORRECT
section form {
  bind Product
  on submit {
    create Product
    toast "Created" success
    navigate "/products"
  }
}
```

## WRONG: Password in columns
```cronus
// WRONG - lint error C031
section table {
  bind User { query all }
  columns "name, email, password, role"
}
```
```cronus
// CORRECT
section table {
  bind User { query all }
  columns "name, email, role"
}
```

## WRONG: Money in reais instead of centavos
```cronus
// WRONG
entity Plan {
  price number default:29.90
}
```
```cronus
// CORRECT - centavos
entity Plan {
  price money! default:2990
}
```

## WRONG: Using location.reload
```cronus
// WRONG - lint error C020
on click {
  location.reload()
}
```
```cronus
// CORRECT
on click {
  refresh
}
```

## WRONG: Data page without auth
```cronus
// WRONG - lint error C030 for shared entities
page "/admin" type:dashboard {
  section table { bind User { query all } }
}
```
```cronus
// CORRECT
page "/admin" type:dashboard requires:auth {
  section table { bind User { query all } }
}
```
