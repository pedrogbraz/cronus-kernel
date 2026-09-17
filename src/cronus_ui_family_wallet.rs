//! Dedicated FamilyWallet renderer (Skiper 21 Family sign-in drawer). DOM
//! matches React: `<div data-slot="family-wallet">` (28rem tall, centred) >
//! the uppercase hint, the `family-wallet-trigger` pill and the drawer
//! (`family-wallet-drawer`: a bottom sheet, max 361px wide, rounded 36px)
//! whose shell holds the round close X and four views —
//! sign-in (five social buttons, the Email / Phone / Passkey segmented
//! control, the field row with its arrow Continue, the `Or` rule and the
//! sky `Connect Wallet` CTA), OTP (back, `Confirm Email` / `Confirm Phone`,
//! the six `input-otp` slots, the green `Verify Code`), passkey (back, the
//! orb with its travelling sky blob, `Waiting for passkey`, Continue) and
//! wallet (back, Metamask / Coinbase / Phantom / Trust Wallet / Other
//! Wallets with their marks, `I Don't Have a Wallet`).
//!
//! Zero JS: the drawer is a native modal `<dialog>` opened by the trigger's
//! `command="show-modal"` (focus trap, Esc, backdrop click via
//! `closedby="any"`) and closed by the X's `command="close"`. The view is a
//! radio group at the top of the views box (`sign-in` checked); Continue,
//! Connect Wallet, Back and the passkey Continue are `<label for>` those
//! radios, so they switch views like React's `setView`. The method is a
//! second radio group whose checked tab shows its pill, its input (`email`
//! / `tel`, `required`, phone with React's pattern) and the matching
//! Continue target (OTP for email / phone, the passkey view for passkey);
//! Continue only accepts pointer input while the field is `:valid`, the
//! kernel's version of `canContinue`. The orb blob rides an `offset-path`
//! round the inset like React's `MovingBorder`. Socials, Verify Code and
//! the wallet rows are React callbacks, rendered `disabled` with the idle
//! look. The typed email / phone cannot be echoed into the OTP heading
//! without JS; the destination line shows the field's placeholder.
//!
//! Inputs: `label "Sign In"` (trigger), `hint:"Click to open sign in"`,
//! `email-placeholder:`, `phone-placeholder:`, `open:true` (drawer starts
//! open), `view:sign-in|otp|passkey|wallet`, `method:email|phone|passkey`.

use crate::cronus_ui_kit::{attr, choice, esc, flag, label_of, truthy};
use crate::parser::ComponentNode;

const GOOGLE: &str = r##"<svg viewBox="0 0 16 16" aria-hidden="true" data-icon="google"><path fill="currentColor" d="M15.545 6.558a9.4 9.4 0 0 1 .139 1.626c0 2.434-.87 4.492-2.384 5.885h.002C11.978 15.292 10.158 16 8 16A8 8 0 1 1 8 0a7.7 7.7 0 0 1 5.352 2.082l-2.284 2.284A4.35 4.35 0 0 0 8 3.166c-2.087 0-3.86 1.408-4.492 3.304a4.8 4.8 0 0 0 0 3.063h.003c.635 1.893 2.405 3.301 4.492 3.301 1.078 0 2.004-.276 2.722-.764h-.003a3.7 3.7 0 0 0 1.599-2.431H8v-3.08z"/></svg>"##;
const DISCORD: &str = r##"<svg viewBox="0 0 16 16" aria-hidden="true" data-icon="discord"><path fill="currentColor" d="M13.545 2.907a13.2 13.2 0 0 0-3.257-1.011.05.05 0 0 0-.052.025c-.141.25-.297.577-.406.833a12.2 12.2 0 0 0-3.658 0 8 8 0 0 0-.412-.833.05.05 0 0 0-.052-.025c-1.125.194-2.22.534-3.257 1.011a.04.04 0 0 0-.021.018C.356 6.024-.213 9.047.066 12.032q.003.022.021.037a13.3 13.2 0 0 0 3.995 2.02.05.05 0 0 0 .056-.019q.463-.63.818-1.329a.05.05 0 0 0-.01-.059l-.018-.011a9 9 0 0 1-1.248-.595.05.05 0 0 1-.02-.066l.015-.019q.127-.095.248-.195a.05.05 0 0 1 .051-.007c2.619 1.196 5.454 1.196 8.041 0a.05.05 0 0 1 .053.007q.121.1.248.195a.05.05 0 0 1-.004.085 8 8 0 0 1-1.249.594.05.05 0 0 0-.03.03.05.05 0 0 0 .003.041c.24.465.515.909.817 1.329a.05.05 0 0 0 .056.019 13.2 13.2 0 0 0 4.001-2.02.05.05 0 0 0 .021-.037c.334-3.451-.559-6.449-2.366-9.106a.03.03 0 0 0-.02-.019m-8.198 7.307c-.789 0-1.438-.724-1.438-1.612s.637-1.613 1.438-1.613c.807 0 1.45.73 1.438 1.613 0 .888-.637 1.612-1.438 1.612m5.316 0c-.788 0-1.438-.724-1.438-1.612s.637-1.613 1.438-1.613c.807 0 1.451.73 1.438 1.613 0 .888-.631 1.612-1.438 1.612"/></svg>"##;
const GITHUB: &str = r##"<svg viewBox="0 0 16 16" aria-hidden="true" data-icon="github"><path fill="currentColor" d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8"/></svg>"##;
const APPLE: &str = r##"<svg viewBox="0 0 16 16" aria-hidden="true" data-icon="apple"><path fill="currentColor" d="M11.182.008C11.148-.03 9.923.023 8.857 1.18c-1.066 1.156-.902 2.482-.878 2.516s1.52.087 2.475-1.258.762-2.391.728-2.43m3.314 11.733c-.048-.096-2.325-1.234-2.113-3.422s1.675-2.789 1.698-2.854-.597-.79-1.254-1.157a3.7 3.7 0 0 0-1.563-.434c-.108-.003-.483-.095-1.254.116-.508.139-1.653.589-1.968.607-.316.018-1.256-.522-2.267-.665-.647-.125-1.333.131-1.824.328-.49.196-1.422.754-2.074 2.237-.652 1.482-.311 3.83-.067 4.56s.625 1.924 1.273 2.796c.576.984 1.34 1.667 1.659 1.899s1.219.386 1.843.067c.502-.308 1.408-.485 1.766-.472.357.013 1.061.154 1.782.539.571.197 1.111.115 1.652-.105.541-.221 1.324-1.059 2.238-2.758q.52-1.185.473-1.282"/></svg>"##;
const X_ICON: &str = r##"<svg viewBox="0 0 16 16" aria-hidden="true" data-icon="x"><path fill="currentColor" d="M12.6.75h2.454l-5.36 6.142L16 15.25h-4.937l-3.867-5.07-4.425 5.07H.316l5.733-6.57L0 .75h5.063l3.495 4.633L12.601.75Zm-.86 13.028h1.36L4.323 2.145H2.865z"/></svg>"##;
const WALLET_ICON: &str = r##"<svg viewBox="0 0 16 16" aria-hidden="true" data-icon="wallet"><path fill="currentColor" d="M12.136.326A1.5 1.5 0 0 1 14 1.78V3h.5A1.5 1.5 0 0 1 16 4.5v9a1.5 1.5 0 0 1-1.5 1.5h-13A1.5 1.5 0 0 1 0 13.5v-9a1.5 1.5 0 0 1 1.432-1.499zM5.562 3H13V1.78a.5.5 0 0 0-.621-.484zM1.5 4a.5.5 0 0 0-.5.5v9a.5.5 0 0 0 .5.5h13a.5.5 0 0 0 .5-.5v-9a.5.5 0 0 0-.5-.5z"/></svg>"##;
const COINBASE_MARK: &str = r##"<svg viewBox="0 0 20 20" aria-hidden="true" data-icon="coinbase"><circle cx="10" cy="10" r="10" fill="#0852FF"/><rect rx="27%" width="20" height="20" fill="#0852FF"/><path fill-rule="evenodd" clip-rule="evenodd" d="M10.0001 17C13.8661 17 17.0001 13.866 17.0001 10C17.0001 6.13401 13.8661 3 10.0001 3C6.13413 3 3.00012 6.13401 3.00012 10C3.00012 13.866 6.13413 17 10.0001 17ZM8.25012 7.71429C7.95427 7.71429 7.71441 7.95414 7.71441 8.25V11.75C7.71441 12.0459 7.95427 12.2857 8.25012 12.2857H11.7501C12.046 12.2857 12.2858 12.0459 12.2858 11.75V8.25C12.2858 7.95414 12.046 7.71429 11.7501 7.71429H8.25012Z" fill="white"/></svg>"##;
const PHANTOM_MARK: &str = r##"<svg viewBox="0 0 593 493" aria-hidden="true" data-icon="phantom"><path fill="#FFFDF8" d="M70.0546 493C145.604 493 202.38 427.297 236.263 375.378C232.142 386.865 229.852 398.351 229.852 409.378C229.852 439.703 247.252 461.297 281.592 461.297C328.753 461.297 379.119 419.946 405.218 375.378C403.386 381.811 402.471 387.784 402.471 393.297C402.471 414.432 414.375 427.757 438.643 427.757C515.108 427.757 592.03 292.216 592.03 173.676C592.03 81.3243 545.327 0 428.112 0C222.069 0 0 251.784 0 414.432C0 478.297 34.3405 493 70.0546 493ZM357.141 163.568C357.141 140.595 369.962 124.514 388.734 124.514C407.049 124.514 419.87 140.595 419.87 163.568C419.87 186.541 407.049 203.081 388.734 203.081C369.962 203.081 357.141 186.541 357.141 163.568ZM455.126 163.568C455.126 140.595 467.947 124.514 486.719 124.514C505.034 124.514 517.855 140.595 517.855 163.568C517.855 186.541 505.034 203.081 486.719 203.081C467.947 203.081 455.126 186.541 455.126 163.568Z"/></svg>"##;
const METAMASK_MARK: &str = r##"<svg viewBox="0 0 256 240" aria-hidden="true" data-icon="metamask"><path fill="#E17726" d="M250.066 0 140.219 81.279l20.427-47.9z"/><path fill="#E27625" d="m6.191.096 89.181 33.289 19.396 48.528zM205.86 172.858l48.551.924-16.968 57.642-59.243-16.311zm-155.721 0 27.557 42.255-59.143 16.312-16.865-57.643z"/><path fill="#E27625" d="m112.131 69.552 1.984 64.083-59.371-2.701 16.888-25.478.214-.245zm31.123-.715 40.9 36.376.212.244 16.888 25.478-59.358 2.7zM79.435 173.044l32.418 25.259-37.658 18.181zm97.136-.004 5.131 43.445-37.553-18.184z"/><path fill="#D5BFB2" d="m144.978 195.922 38.107 18.452-35.447 16.846.368-11.134zm-33.967.008-2.909 23.974.239 11.303-35.53-16.833z"/><path fill="#233447" d="m100.007 141.999 9.958 20.928-33.903-9.932zm55.985.002 24.058 10.994-34.014 9.929z"/><path fill="#CC6228" d="m82.026 172.83-5.48 45.04-29.373-44.055zm91.95.001 34.854.984-29.483 44.057zm28.136-44.444-25.365 25.851-19.557-8.937-9.363 19.684-6.138-33.849zm-148.237 0 60.435 2.749-6.139 33.849-9.365-19.681-19.453 8.935z"/><path fill="#E27525" d="m52.166 123.082 28.698 29.121.994 28.749zm151.697-.052-29.746 57.973 1.12-28.8zm-90.956 1.826 1.155 7.27 2.854 18.111-1.835 55.625-8.675-44.685-.003-.462zm30.171-.101 6.521 35.96-.003.462-8.697 44.797-.344-11.205-1.357-44.862z"/><path fill="#F5841F" d="m177.788 151.046-.971 24.978-30.274 23.587-6.12-4.324 6.86-35.335zm-99.471 0 30.399 8.906 6.86 35.335-6.12 4.324-30.275-23.589z"/><path fill="#C0AC9D" d="m67.018 208.858 38.732 18.352-.164-7.837 3.241-2.845h38.334l3.358 2.835-.248 7.831 38.487-18.29-18.728 15.476-22.645 15.553h-38.869l-22.63-15.617z"/><path fill="#161616" d="m142.204 193.479 5.476 3.869 3.209 25.604-4.644-3.921h-36.476l-4.556 4 3.104-25.681 5.478-3.871z"/><path fill="#763E1A" d="M242.814 2.25 256 41.807l-8.235 39.997 5.864 4.523-7.935 6.054 5.964 4.606-7.897 7.191 4.848 3.511-12.866 15.026-52.77-15.365-.457-.245-38.027-32.078zm-229.628 0 98.326 72.777-38.028 32.078-.457.245-52.77 15.365-12.866-15.026 4.844-3.508-7.892-7.194 5.952-4.601-8.054-6.071L6.085 41.809 0 41.809z"/><path fill="#F5841F" d="m180.392 103.99 55.913 16.279 18.165 55.986h-47.924l-33.02.416 24.014-46.808zm-104.784 0-17.151 25.873 24.017 46.808-33.005-.416H1.631l18.063-55.985zm87.776-70.878-15.639 42.239-3.319 57.06-1.27 17.885-.101 45.688h-30.111l-.098-45.602-1.274-17.986-3.32-57.045-15.637-42.239z"/></svg>"##;
fn trust_mark(gid: &str) -> String {
    format!(
        r##"<svg viewBox="0 0 24 24" aria-hidden="true" data-icon="trust"><path fill="#0500FF" d="M3.9 5.6 12 3v18c-5.786-2.4-8.1-7-8.1-9.6z"/><path fill="url(#{gid})" d="M20.1 5.6 12 3v18c5.786-2.4 8.1-7 8.1-9.6z"/><defs><linearGradient id="{gid}" x1="17.948" x2="11.967" y1="1.74" y2="20.797"><stop offset=".02" stop-color="#00F"/><stop offset=".08" stop-color="#0094FF"/><stop offset=".16" stop-color="#48FF91"/><stop offset=".42" stop-color="#0094FF"/><stop offset=".68" stop-color="#0038FF"/><stop offset=".9" stop-color="#0500FF"/></linearGradient></defs></svg>"##
    )
}

const SOCIALS: [(&str, &str); 5] = [
    ("Sign in with Google", GOOGLE),
    ("Sign in with Discord", DISCORD),
    ("Sign in with GitHub", GITHUB),
    ("Sign in with Apple", APPLE),
    ("Sign in with Farcaster", X_ICON),
];
const VIEWS: &[&str] = &["sign-in", "otp", "passkey", "wallet"];
const METHODS: &[&str] = &["email", "phone", "passkey"];

pub fn render(comp: &ComponentNode) -> String {
    let trigger = label_of(comp);
    let hint = match attr(comp, "hint") {
        Some(h) => esc(h.trim()),
        None => "Click to open sign in".to_string(),
    };
    let hint_html = if hint.is_empty() {
        String::new()
    } else {
        format!("<p>{hint}</p>")
    };
    let email_ph = attr(comp, "email-placeholder")
        .map(esc)
        .unwrap_or_else(|| "yo@gxuri.me".to_string());
    let phone_ph = attr(comp, "phone-placeholder")
        .map(esc)
        .unwrap_or_else(|| "+1 (555) 123-4567".to_string());
    let view = choice(comp, "view", VIEWS).unwrap_or("sign-in");
    let method = choice(comp, "method", METHODS).unwrap_or("email");
    let open = flag(comp, "open") || attr(comp, "default-open").is_some_and(|v| truthy(v));
    let id = crate::cronus_ui_kit::instance_id(comp, "family-wallet");
    let trigger_id = format!("{id}-trigger");
    let dialog_id = format!("{id}-drawer");
    let view_name = format!("{id}-view");
    let method_name = format!("{id}-method");
    let gid = format!("{id}-trust");
    let plus = crate::cronus_ui_icons::svg_or_empty("plus");
    let left = crate::cronus_ui_icons::svg_or_empty("chevron-left");
    let right = crate::cronus_ui_icons::svg_or_empty("arrow-right");
    let fingerprint = crate::cronus_ui_receive_button::fingerprint_icon();

    let radios: String = VIEWS
        .iter()
        .map(|v| {
            let checked = if *v == view { " checked" } else { "" };
            let name = match *v {
                "otp" => "Confirm",
                "passkey" => "Passkey",
                "wallet" => "Connect Wallet",
                _ => "Sign In",
            };
            format!(
                "<input type=\"radio\" name=\"{view_name}\" id=\"{id}-{v}\" aria-label=\"{name}\"{checked}>"
            )
        })
        .collect();
    let socials: String = SOCIALS
        .iter()
        .map(|(label, icon)| {
            format!("<button type=\"button\" aria-label=\"{label}\" disabled>{icon}</button>")
        })
        .collect();
    let methods: String = METHODS
        .iter()
        .map(|m| {
            let checked = if *m == method { " checked" } else { "" };
            let label = match *m {
                "phone" => "Phone",
                "passkey" => "Passkey",
                _ => "Email",
            };
            format!(
                "<label><input type=\"radio\" name=\"{method_name}\" aria-label=\"Select {label}\"{checked}><span class=\"pill\"></span><span>{label}</span></label>"
            )
        })
        .collect();
    let back = |target: &str| {
        format!("<label for=\"{id}-{target}\" class=\"round\" aria-label=\"Back\">{left}</label>")
    };
    let sign_in = format!(
        "<section class=\"view sign-in\"><h2><span>Sign In</span></h2><div class=\"stack\"><div class=\"auth\"><div class=\"socials\">{socials}</div><div class=\"methods\"><div>{methods}</div></div><div class=\"field\"><div><input type=\"email\" class=\"email\" placeholder=\"{email_ph}\" required aria-label=\"Email\"><input type=\"tel\" class=\"phone\" placeholder=\"{phone_ph}\" required pattern=\"[+]?[0-9\\s()-]{{10,}}\" aria-label=\"Phone\"><div class=\"passkey-row\">{fingerprint}<span>Login with passkey</span></div></div><label for=\"{id}-otp\" class=\"continue\" aria-label=\"Continue\">{right}</label><label for=\"{id}-passkey\" class=\"continue\" aria-label=\"Continue\">{right}</label></div></div><div class=\"or\"><div><span></span></div><div><span>Or</span></div></div><label for=\"{id}-wallet\" class=\"connect\">{WALLET_ICON}Connect Wallet</label></div></section>"
    );
    let otp_slots: String = (0..6)
        .map(|_| "<div data-slot=\"input-otp-slot\"></div>")
        .collect();
    let otp = format!(
        "<section class=\"view otp\"><div class=\"bar\">{}<h2><span class=\"for-email\">Confirm Email</span><span class=\"for-phone\">Confirm Phone</span></h2><span class=\"round ghost\"></span></div><div class=\"stack\"><div><p>Enter the verification code sent to</p><p class=\"dest\"><span class=\"for-email\">{email_ph}</span><span class=\"for-phone\">{phone_ph}</span></p></div><div class=\"stack\"><div data-input-otp-container=\"true\"><div data-slot=\"input-otp-group\">{otp_slots}</div><div><input data-slot=\"input-otp\" autocomplete=\"one-time-code\" aria-label=\"Verification code\" inputmode=\"numeric\" maxlength=\"6\" value=\"\" /></div></div><button type=\"button\" class=\"verify\" disabled>Verify Code</button></div></div></section>",
        back("sign-in")
    );
    let passkey = format!(
        "<section class=\"view passkey\"><div class=\"bar\">{}<h2>Passkey</h2><span class=\"round ghost\"></span></div><div class=\"stack center\"><div class=\"orb\"><div class=\"ring\"><i></i></div><div class=\"core\">{fingerprint}</div></div><div><h3>Waiting for passkey</h3><p>Please follow prompts to verify your passkey.</p></div><label for=\"{id}-sign-in\" class=\"continue-wide\">Continue</label></div></section>",
        back("sign-in")
    );
    let wallets = [
        ("Metamask", METAMASK_MARK.to_string()),
        ("Coinbase", COINBASE_MARK.to_string()),
        (
            "Phantom",
            format!("<span class=\"phantom\">{PHANTOM_MARK}</span>"),
        ),
        ("Trust Wallet", trust_mark(&gid)),
    ];
    let wallet_rows: String = wallets
        .iter()
        .map(|(name, mark)| {
            format!("<button type=\"button\" disabled><span>{name}</span>{mark}</button>")
        })
        .collect();
    let wallet = format!(
        "<section class=\"view wallet\"><div class=\"bar\">{}<h2>Connect Wallet</h2><span class=\"round ghost\"></span></div><div class=\"list\">{wallet_rows}<button type=\"button\" disabled><span class=\"other\"><span>Other Wallets</span><span class=\"count\">350+</span></span><span class=\"other-mark\">{WALLET_ICON}</span></button><button type=\"button\" class=\"none\" disabled>{WALLET_ICON}I Don't Have a Wallet</button></div></section>",
        back("sign-in")
    );
    let open_attr = if open { " open" } else { "" };
    format!(
        "<div data-slot=\"family-wallet\">{hint_html}<button type=\"button\" id=\"{trigger_id}\" data-slot=\"family-wallet-trigger\" commandfor=\"{dialog_id}\" command=\"show-modal\" aria-haspopup=\"dialog\">{trigger}</button><dialog id=\"{dialog_id}\" data-slot=\"family-wallet-drawer\" aria-labelledby=\"{dialog_id}-title\" closedby=\"any\"{open_attr}><h2 id=\"{dialog_id}-title\" class=\"sr-only\">Sign In</h2><div class=\"shell\"><button type=\"button\" class=\"round close\" aria-label=\"Close\" commandfor=\"{dialog_id}\" command=\"close\">{plus}</button><div class=\"views\">{radios}{sign_in}{otp}{passkey}{wallet}</div></div></dialog></div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::stub_renderer_gate::{dedicated_fn_name, renderer_kind, RendererKind};
    use crate::cronus_ui_kit::{reset_instance_ids, stub};

    fn wallet() -> ComponentNode {
        reset_instance_ids();
        stub("family-wallet", "Sign In")
    }

    fn reject_js(html: &str) {
        for bad in [
            "<script", "style=", "onclick", "onmouse", "onkey", "<canvas", "popover", "<details",
        ] {
            assert!(!html.contains(bad), "{bad} in {html}");
        }
    }

    #[test]
    fn docs_sign_in_drawer_dom() {
        let html = render(&wallet());
        assert!(html.starts_with("<div data-slot=\"family-wallet\"><p>Click to open sign in</p><button type=\"button\" id=\"cui-family-wallet-family-wallet-trigger\" data-slot=\"family-wallet-trigger\" commandfor=\"cui-family-wallet-family-wallet-drawer\" command=\"show-modal\" aria-haspopup=\"dialog\">Sign In</button><dialog id=\"cui-family-wallet-family-wallet-drawer\" data-slot=\"family-wallet-drawer\" aria-labelledby=\"cui-family-wallet-family-wallet-drawer-title\" closedby=\"any\"><h2 id=\"cui-family-wallet-family-wallet-drawer-title\" class=\"sr-only\">Sign In</h2><div class=\"shell\"><button type=\"button\" class=\"round close\" aria-label=\"Close\" commandfor=\"cui-family-wallet-family-wallet-drawer\" command=\"close\"><svg"));
        // Views: radio group + four sections, sign-in checked.
        assert!(html.contains("<div class=\"views\"><input type=\"radio\" name=\"cui-family-wallet-family-wallet-view\" id=\"cui-family-wallet-family-wallet-sign-in\" aria-label=\"Sign In\" checked><input type=\"radio\" name=\"cui-family-wallet-family-wallet-view\" id=\"cui-family-wallet-family-wallet-otp\" aria-label=\"Confirm\"><input type=\"radio\" name=\"cui-family-wallet-family-wallet-view\" id=\"cui-family-wallet-family-wallet-passkey\" aria-label=\"Passkey\"><input type=\"radio\" name=\"cui-family-wallet-family-wallet-view\" id=\"cui-family-wallet-family-wallet-wallet\" aria-label=\"Connect Wallet\"><section class=\"view sign-in\"><h2><span>Sign In</span></h2>"));
        assert_eq!(html.matches("<section class=\"view ").count(), 4);
        // Socials (disabled callbacks), methods, field, continue targets, connect.
        assert_eq!(html.matches("aria-label=\"Sign in with ").count(), 5);
        assert!(html.contains("<button type=\"button\" aria-label=\"Sign in with Google\" disabled><svg viewBox=\"0 0 16 16\" aria-hidden=\"true\" data-icon=\"google\">"));
        assert!(html.contains("<label><input type=\"radio\" name=\"cui-family-wallet-family-wallet-method\" aria-label=\"Select Email\" checked><span class=\"pill\"></span><span>Email</span></label><label><input type=\"radio\" name=\"cui-family-wallet-family-wallet-method\" aria-label=\"Select Phone\"><span class=\"pill\"></span><span>Phone</span></label><label><input type=\"radio\" name=\"cui-family-wallet-family-wallet-method\" aria-label=\"Select Passkey\"><span class=\"pill\"></span><span>Passkey</span></label>"));
        assert!(html.contains("<input type=\"email\" class=\"email\" placeholder=\"yo@gxuri.me\" required aria-label=\"Email\"><input type=\"tel\" class=\"phone\" placeholder=\"+1 (555) 123-4567\" required pattern=\"[+]?[0-9\\s()-]{10,}\" aria-label=\"Phone\"><div class=\"passkey-row\"><svg"));
        assert!(html.contains("<label for=\"cui-family-wallet-family-wallet-otp\" class=\"continue\" aria-label=\"Continue\"><svg"));
        assert!(html.contains("<label for=\"cui-family-wallet-family-wallet-passkey\" class=\"continue\" aria-label=\"Continue\"><svg"));
        assert!(html.contains("<div class=\"or\"><div><span></span></div><div><span>Or</span></div></div><label for=\"cui-family-wallet-family-wallet-wallet\" class=\"connect\"><svg viewBox=\"0 0 16 16\" aria-hidden=\"true\" data-icon=\"wallet\">"));
        // OTP view.
        assert!(html.contains("<section class=\"view otp\"><div class=\"bar\"><label for=\"cui-family-wallet-family-wallet-sign-in\" class=\"round\" aria-label=\"Back\"><svg"));
        assert!(html.contains("<h2><span class=\"for-email\">Confirm Email</span><span class=\"for-phone\">Confirm Phone</span></h2><span class=\"round ghost\"></span></div>"));
        assert_eq!(
            html.matches("<div data-slot=\"input-otp-slot\"></div>")
                .count(),
            6
        );
        assert!(
            html.contains("<button type=\"button\" class=\"verify\" disabled>Verify Code</button>")
        );
        // Passkey view.
        assert!(html.contains(
            "<div class=\"orb\"><div class=\"ring\"><i></i></div><div class=\"core\"><svg"
        ));
        assert!(html.contains("<h3>Waiting for passkey</h3><p>Please follow prompts to verify your passkey.</p></div><label for=\"cui-family-wallet-family-wallet-sign-in\" class=\"continue-wide\">Continue</label>"));
        // Wallet view.
        assert!(html.contains("<button type=\"button\" disabled><span>Metamask</span><svg viewBox=\"0 0 256 240\" aria-hidden=\"true\" data-icon=\"metamask\">"));
        assert!(html.contains("<span>Coinbase</span><svg viewBox=\"0 0 20 20\" aria-hidden=\"true\" data-icon=\"coinbase\">"));
        assert!(html
            .contains("<span>Phantom</span><span class=\"phantom\"><svg viewBox=\"0 0 593 493\""));
        assert!(html.contains("<span>Trust Wallet</span><svg viewBox=\"0 0 24 24\" aria-hidden=\"true\" data-icon=\"trust\">"));
        assert!(html.contains("fill=\"url(#cui-family-wallet-family-wallet-trust)\""));
        assert!(html.contains("<linearGradient id=\"cui-family-wallet-family-wallet-trust\""));
        assert!(html.contains("<span class=\"other\"><span>Other Wallets</span><span class=\"count\">350+</span></span>"));
        assert!(html.ends_with(
            "I Don't Have a Wallet</button></div></section></div></div></dialog></div>"
        ));
        reject_js(&html);
    }

    #[test]
    fn view_method_open_and_placeholders() {
        let mut c = wallet();
        c.props.insert("view".into(), "wallet".into());
        c.props.insert("method".into(), "phone".into());
        c.props.insert("open".into(), "true".into());
        c.props
            .insert("email-placeholder".into(), "you@cronus.dev".into());
        let html = render(&c);
        assert!(html.contains("aria-label=\"Sign In\"><input"));
        assert!(html.contains("aria-label=\"Connect Wallet\" checked>"));
        assert!(html.contains("aria-label=\"Select Phone\" checked>"));
        assert!(!html.contains("aria-label=\"Select Email\" checked>"));
        assert!(html.contains("closedby=\"any\" open>"));
        assert!(html.contains("placeholder=\"you@cronus.dev\""));
        assert!(html.contains("<p class=\"dest\"><span class=\"for-email\">you@cronus.dev</span>"));
    }

    #[test]
    fn escapes_hostile_input() {
        let mut c = wallet();
        c.items[0].text = "<b>Go</b> \"q\"".into();
        c.props.insert("hint".into(), "<i>h</i>".into());
        c.props
            .insert("email-placeholder".into(), "\" onfocus=\"x".into());
        let html = render(&c);
        assert!(html.contains("<p>&lt;i&gt;h&lt;/i&gt;</p>"));
        assert!(
            html.contains("aria-haspopup=\"dialog\">&lt;b&gt;Go&lt;/b&gt; &quot;q&quot;</button>")
        );
        assert!(html.contains("placeholder=\"&quot; onfocus=&quot;x\""));
        reject_js(&html);
    }

    #[test]
    fn chrome_is_token_only_or_component_painted() {
        let css = include_str!("cronus_ui_css/family-wallet.css");
        assert!(css.contains("[data-slot=\"family-wallet\"] {"));
        assert!(css.contains("[data-slot=\"family-wallet-drawer\"]:modal"));
        assert!(css.contains("max-width: 22.5625rem"));
        assert!(css.contains("border-radius: 36px"));
        assert!(css.contains("--cui-family-sky"));
        assert!(css.contains("--cui-family-green"));
        assert!(css.contains(".views > input:nth-of-type(4):checked ~ .view:nth-of-type(4)"));
        assert!(css.contains(".auth:has(.methods label:nth-child(2) > input:checked)"));
        assert!(css.contains(".email:valid"));
        assert!(css.contains("offset-path: inset(0 round 30%)"));
        assert!(css.contains("270ms cubic-bezier(0.25, 1, 0.5, 1)"));
        assert!(css.contains("[data-slot=\"family-wallet\"] [data-slot=\"input-otp-slot\"]"));
    }

    #[test]
    fn registered_as_dedicated() {
        let c = wallet();
        assert_eq!(crate::cronus_ui_widgets::render(&c).unwrap(), {
            reset_instance_ids();
            render(&c)
        });
        assert_eq!(
            dedicated_fn_name("family-wallet"),
            Some("cronus_ui_family_wallet::render")
        );
        assert_eq!(
            renderer_kind("family-wallet"),
            RendererKind::Dedicated("cronus_ui_family_wallet::render")
        );
    }
}
