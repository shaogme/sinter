use sinter_core::{ContentNode, Post, SiteMetaData, SitePostMetadata};
use sinter_theme_sdk::{Children, Theme};
use sinter_ui::dom::tag::*;
use sinter_ui::dom::suspense::suspense;
use sinter_ui::dom::view::{AnyView, IntoAnyView};
use sinter_ui::prelude::*;

#[derive(Clone, Debug)]
pub struct DefaultLightTheme;

impl Theme for DefaultLightTheme {
    fn render_layout(
        &self,
        children: Children,
        site_data: ReadSignal<Option<SiteMetaData>>,
    ) -> AnyView {
        let site_title = move || {
            site_data
                .get()
                .flatten()
                .map(|d| d.title)
                .unwrap_or_else(|| "Sinter".to_string())
        };

        div()
            .class("flex flex-col min-h-screen font-sans text-slate-800 relative bg-slate-50")
            .child((
                // --- Aurora Background ---
                div().class("aurora-bg").child((
                    div().class("blob"),
                    div().class("blob"),
                    div().class("blob"),
                )),
                div().class("overlay"),
                // --- SVG Filter 注入 (完全对应 index.html 参数) ---
                svg()
                    .attr("xmlns", "http://www.w3.org/2000/svg")
                    .style("position: absolute; width: 0; height: 0; overflow: hidden; pointer-events: none;")
                    .attr("aria-hidden", "true")
                    .child(defs().child(
                        filter()
                            .id("glass-distortion")
                            .attr("x", "0%")
                            .attr("y", "0%")
                            .attr("width", "100%")
                            .attr("height", "100%")
                            .attr("filterUnits", "objectBoundingBox")
                            .child((
                                fe_turbulence()
                                    .attr("type", "fractalNoise")
                                    .attr("baseFrequency", "0.01 0.01")
                                    .attr("numOctaves", "1")
                                    .attr("seed", "5")
                                    .attr("result", "turbulence"),
                                fe_component_transfer()
                                    .attr("in", "turbulence")
                                    .attr("result", "mapped")
                                    .child((
                                        fe_func_r()
                                            .attr("type", "gamma")
                                            .attr("amplitude", "1")
                                            .attr("exponent", "10")
                                            .attr("offset", "0.5"),
                                        fe_func_g()
                                            .attr("type", "gamma")
                                            .attr("amplitude", "0")
                                            .attr("exponent", "1")
                                            .attr("offset", "0"),
                                        fe_func_b()
                                            .attr("type", "gamma")
                                            .attr("amplitude", "0")
                                            .attr("exponent", "1")
                                            .attr("offset", "0.5"),
                                    )),
                                fe_gaussian_blur()
                                    .attr("in", "turbulence")
                                    .attr("stdDeviation", "3")
                                    .attr("result", "softMap"),
                                fe_specular_lighting()
                                    .attr("in", "softMap")
                                    .attr("surfaceScale", "5")
                                    .attr("specularConstant", "1")
                                    .attr("specularExponent", "100")
                                    .attr("lighting-color", "white")
                                    .attr("result", "specLight")
                                    .child(
                                        fe_point_light()
                                            .attr("x", "-200")
                                            .attr("y", "-200")
                                            .attr("z", "300"),
                                    ),
                                fe_composite()
                                    .attr("in", "specLight")
                                    .attr("operator", "arithmetic")
                                    .attr("k1", "0")
                                    .attr("k2", "1")
                                    .attr("k3", "1")
                                    .attr("k4", "0")
                                    .attr("result", "litImage"),
                                fe_displacement_map()
                                    .attr("in", "SourceGraphic")
                                    .attr("in2", "softMap")
                                    .attr("scale", "150")
                                    .attr("xChannelSelector", "R")
                                    .attr("yChannelSelector", "G")
                                    .attr("result", "distorted"),
                                fe_composite()
                                    .attr("in", "specLight")
                                    .attr("in2", "distorted")
                                    .attr("operator", "in")
                                    .attr("result", "final"),
                            )),
                    )),
                // Navbar
                render_navbar(site_title),
                // Main Content
                main().class("flex-grow w-full").child(children()),
                // Footer
                render_footer(),
            ))
            .into_any()
    }

    fn render_home(&self) -> AnyView {
        let site_meta_r = sinter_theme_sdk::use_site_meta();
        let page_data_r = sinter_theme_sdk::use_page_data();
        let current_page_s = sinter_theme_sdk::use_current_page();

        let theme_fallback = self.clone();
        let theme_fallback_clone = theme_fallback.clone();

        suspense()
            .fallback(move || theme_fallback.render_loading())
            .children(move || {
                let site_meta_res = site_meta_r.and_then(|r| r.get()).and_then(|r| r.ok());
                let page_data_res = page_data_r
                    .clone()
                    .and_then(|r| r.get().and_then(|res| res.ok()));

                if let (Some(site_meta), Some(page_data)) = (site_meta_res, page_data_res) {
                    let posts = page_data.posts;
                    let title = site_meta.title.clone();
                    let subtitle = site_meta.subtitle.clone();
                    let description = site_meta.description.clone();
                    let total_pages = site_meta.total_pages;

                    let search = current_page_s.get().unwrap_or(1);

                    let posts_clone = posts.clone();

                    div()
                        .class("flex flex-col w-full")
                        .child((
                            // Hero Section
                            div().class("hero min-h-screen relative").child(
                                div().class("hero-content text-center text-slate-900 z-10 w-full").child(
                                    div().class("w-full max-w-7xl mx-auto flex flex-col items-center animate-fade-in-up").child((
                                        // --- Liquid Glass Component ---
                                        div().class("mb-12").child(
                                            div().class("liquidGlass-wrapper").child((
                                                div().class("liquidGlass-effect"),
                                                div().class("liquidGlass-tint"),
                                                div().class("liquidGlass-shine"),
                                                div().class("liquidGlass-text").child(
                                                    h1().class("text-6xl md:text-8xl lg:text-9xl font-black tracking-tighter leading-none")
                                                        .text(title)
                                                ),
                                            ))
                                        ),
                                        // ------------------------------
                                        div().class("text-center text-slate-800 space-y-6 max-w-2xl mx-auto px-4").child((
                                            h2().class("text-2xl md:text-4xl font-bold opacity-90 drop-shadow-sm").text(subtitle),
                                            p().class("text-lg md:text-2xl font-medium opacity-80").text(description),
                                        )),
                                        // Bounce arrow
                                        div().class("absolute bottom-10 left-1/2 -translate-x-1/2 animate-bounce").child(
                                            svg().class("h-10 w-10 text-slate-700").attr("fill", "none").attr("viewBox", "0 0 24 24").attr("stroke", "currentColor").child(
                                                path().attr("stroke-linecap", "round").attr("stroke-linejoin", "round").attr("stroke-width", "2").attr("d", "M19 14l-7 7m0 0l-7-7m7 7V3")
                                            )
                                        )
                                    ))
                                )
                            ),
                            // Posts Grid
                            div().class("py-20 px-4 min-h-[50vh]").child(
                                div().class("container mx-auto max-w-5xl").child((
                                    For::new(
                                        move || Ok(posts_clone.clone()),
                                        |post| post.metadata.id.clone(),
                                        |post| render_post_card(post, false) // false for home
                                    ),
                                    // Pagination Controls
                                    render_pagination(search, total_pages, false)
                                ))
                            )
                        ))
                        .into_any()
                } else {
                     theme_fallback_clone.render_loading()
                }
            })
            .into_any()
    }

    fn render_archive(&self) -> AnyView {
        let site_meta_r = sinter_theme_sdk::use_site_meta();
        let page_data_r = sinter_theme_sdk::use_page_data();
        let current_page_s = sinter_theme_sdk::use_current_page();

        let theme_fallback = self.clone();
        let theme_fallback_clone = theme_fallback.clone();

        suspense()
            .fallback(move || theme_fallback.render_loading())
            .children(move || {
                let site_meta_res = site_meta_r.and_then(|r| r.get()).and_then(|r| r.ok());
                let page_data_res = page_data_r
                    .clone()
                    .and_then(|r| r.get().and_then(|res| res.ok()));

                if let (Some(site_meta), Some(page_data)) = (site_meta_res, page_data_res) {
                    let posts = page_data.posts;
                    let title = site_meta.title.clone();
                    let subtitle = site_meta.subtitle.clone();
                    let description = site_meta.description.clone();
                    let total_pages = site_meta.total_pages;

                    let search = current_page_s.get().unwrap_or(1);

                    let posts_clone = posts.clone();

                    div()
                        .class("flex flex-col w-full")
                        .child((
                            // Hero Section
                            div().class("hero min-h-screen relative").child(
                                div().class("hero-content text-center text-slate-900 z-10 w-full").child(
                                    div().class("w-full max-w-7xl mx-auto flex flex-col items-center animate-fade-in-up").child((
                                        // --- Liquid Glass Component ---
                                        div().class("mb-12").child(
                                            div().class("liquidGlass-wrapper").child((
                                                div().class("liquidGlass-effect"),
                                                div().class("liquidGlass-tint"),
                                                div().class("liquidGlass-shine"),
                                                div().class("liquidGlass-text").child(
                                                    h1().class("text-6xl md:text-8xl lg:text-9xl font-black tracking-tighter leading-none")
                                                        .text(title)
                                                ),
                                            ))
                                        ),
                                        // ------------------------------
                                        div().class("text-center text-slate-800 space-y-6 max-w-2xl mx-auto px-4").child((
                                            h2().class("text-2xl md:text-4xl font-bold opacity-90 drop-shadow-sm").text(subtitle),
                                            p().class("text-lg md:text-2xl font-medium opacity-80").text(description),
                                        )),
                                        // Bounce arrow
                                        div().class("absolute bottom-10 left-1/2 -translate-x-1/2 animate-bounce").child(
                                            svg().class("h-10 w-10 text-slate-700").attr("fill", "none").attr("viewBox", "0 0 24 24").attr("stroke", "currentColor").child(
                                                path().attr("stroke-linecap", "round").attr("stroke-linejoin", "round").attr("stroke-width", "2").attr("d", "M19 14l-7 7m0 0l-7-7m7 7V3")
                                            )
                                        )
                                    ))
                                )
                            ),
                            // Posts Grid
                            div().class("py-20 px-4 min-h-[50vh]").child(
                                div().class("container mx-auto max-w-5xl").child((
                                    For::new(
                                        move || Ok(posts_clone.clone()),
                                        |post| post.metadata.id.clone(),
                                        |post| render_post_card(post, true) // true for archive
                                    ),
                                    // Pagination Controls
                                    render_pagination(search, total_pages, true)
                                ))
                            )
                        ))
                        .into_any()
                } else {
                     theme_fallback_clone.render_loading()
                }
            })
            .into_any()
    }

    fn render_post(&self, post: Post) -> AnyView {
        let content_ast = post.content_ast.clone();
        
        div()
            .class("pt-24 lg:pt-32 pb-20 px-4")
            .child(
                article()
                    .class("max-w-4xl mx-auto animate-fade-in relative")
                    .child((
                        // Glass Container
                        div().class("absolute inset-0 -mx-4 sm:-mx-12 bg-white/60 backdrop-blur-xl rounded-[2.5rem] border border-white/40 shadow-2xl z-0"),
                        // Content Wrapper
                        div().class("relative z-10 p-4 sm:p-12").child((
                            header().class("text-center mb-16 space-y-6").child((
                                h1().class("text-4xl md:text-5xl lg:text-6xl font-black text-slate-900 leading-tight drop-shadow-sm")
                                    .text(post.metadata.title.clone()),
                                div().class("flex flex-wrap items-center justify-center gap-4 text-sm font-medium text-slate-600").child((
                                    time().class("px-4 py-1.5 rounded-full bg-white/40 border border-slate-200 backdrop-blur-sm")
                                        .text(format_date_long(&post.metadata.date)),
                                    div().class("flex gap-2").child(
                                        For::new(
                                            move || Ok(post.metadata.tags.clone()),
                                            |tag| tag.clone(),
                                            |tag| span().class("px-3 py-1 rounded-full bg-primary/10 text-primary border border-primary/10 backdrop-blur-sm uppercase tracking-wider text-xs").text(tag)
                                        )
                                    )
                                ))
                            )),
                            div().class("prose prose-lg mx-auto max-w-none prose-headings:text-slate-900 prose-p:text-slate-800 prose-a:text-blue-600 prose-blockquote:border-l-primary prose-code:text-primary")
                                .child(
                                    For::new(
                                        move || Ok(content_ast.iter().enumerate().map(|(i, n)| (i, n.clone())).collect::<Vec<_>>()),
                                        |(i, _)| *i,
                                        |(_, node)| render_node(node)
                                    )
                                ),
                            div().class("mt-20 pt-10 border-t border-slate-200 text-center").child(
                                a().attr("href", "/")
                                    .class("btn btn-ghost hover:bg-black/5 text-slate-800 gap-3 rounded-full px-8")
                                    .child((
                                        svg().class("h-5 w-5").attr("fill", "none").attr("viewBox", "0 0 24 24").attr("stroke", "currentColor").child(
                                            path().attr("stroke-linecap", "round").attr("stroke-linejoin", "round").attr("stroke-width", "2").attr("d", "M10 19l-7-7m0 0l7-7m-7 7h18")
                                        ),
                                        "Back to Home"
                                    ))
                            )
                        ))
                    ))
            )
            .into_any()
    }

    fn render_post_loading(&self) -> AnyView {
        div()
            .class("flex justify-center items-center min-h-screen pt-20")
            .child(span().class("loading loading-spinner loading-lg text-primary"))
            .into_any()
    }

    fn render_loading(&self) -> AnyView {
        div()
            .class("flex justify-center items-center h-full w-full min-h-[50vh]")
            .child(span().class("loading loading-dots loading-lg text-secondary"))
            .into_any()
    }

    fn render_post_not_found(&self) -> AnyView {
        div()
            .class("hero min-h-screen pt-16")
            .child(
                div().class("hero-content text-center").child(
                    div().class("max-w-md space-y-8").child((
                        h1().class("text-9xl font-black text-slate-300").text("404"),
                        h2().class("text-4xl font-bold text-slate-900").text("Page Not Found"),
                        p().class("text-lg text-slate-700").text("The content you're looking for seems to have been moved or deleted."),
                        a().attr("href", "/").class("btn btn-primary btn-lg min-w-[200px]").text("Return Home")
                    ))
                )
            )
            .into_any()
    }

    fn render_error(&self, message: String) -> AnyView {
        div()
            .class("flex justify-center items-center h-full min-h-[50vh] p-4")
            .child(
                div().class("alert alert-error shadow-lg rounded-xl max-w-lg").child((
                    svg().class("stroke-current flex-shrink-0 h-6 w-6").attr("fill", "none").attr("viewBox", "0 0 24 24").child(
                        path().attr("stroke-linecap", "round").attr("stroke-linejoin", "round").attr("stroke-width", "2").attr("d", "M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z")
                    ),
                    div().child((
                        h3().class("font-bold").text("Error loading content"),
                        div().class("text-xs break-words mt-1").text(message)
                    ))
                ))
            )
            .into_any()
    }
}

// --- Helpers ---

fn render_navbar<F>(site_title: F) -> Element
where
    F: Fn() -> String + 'static,
{
    nav().class("navbar fixed top-0 z-50 transition-all duration-300 hover:bg-black/5 hover:shadow-sm text-slate-800 has-[.scrolled]:bg-white/60 backdrop-blur-md border-b border-black/5")
        .child(
            div().class("container mx-auto px-4 flex items-center").child((
                div().class("flex-1").child(
                    a().attr("href", "/").class("btn btn-ghost text-2xl font-black tracking-tighter hover:scale-105 transition-transform text-slate-900 drop-shadow-sm")
                        .child(site_title)
                ),
                div().class("flex-none hidden md:block").child(
                    ul().class("menu menu-horizontal px-1 font-medium text-slate-700").child((
                        li().child(a().attr("href", "/").class("hover:bg-black/5 hover:text-slate-900 transition-all rounded-lg").text("Home")),
                        li().child(a().attr("href", "/archives").class("hover:bg-black/5 hover:text-slate-900 transition-all rounded-lg").text("Archives"))
                    ))
                ),
                div().class("flex-none").child(
                    div().class("dropdown dropdown-end").child((
                        div().attr("tabindex", "0").attr("role", "button").class("btn btn-ghost hover:bg-black/5 text-slate-800 rounded-btn gap-2").child((
                            svg().attr("xmlns", "http://www.w3.org/2000/svg").attr("fill", "none").attr("viewBox", "0 0 24 24").attr("stroke-width", "1.5").attr("stroke", "currentColor").class("w-5 h-5").child(
                                path().attr("stroke-linecap", "round").attr("stroke-linejoin", "round").attr("d", "M4.098 19.902a3.75 3.75 0 0 0 5.304 0l4.56-4.56a3.75 3.75 0 0 0-5.304-5.304l-4.56 4.56a3.75 3.75 0 0 0 0 5.304Zm1.706-9.191a1.5 1.5 0 0 1 2.122 0l2.122 2.121a1.5 1.5 0 0 1 0 2.122l-2.122 2.122a1.5 1.5 0 0 1-2.122 0l-2.122-2.122a1.5 1.5 0 0 1 0-2.122l2.122-2.122Zm9.706-5.464 1.258 1.258a1.5 1.5 0 0 1 0 2.122l-1.258 1.258a1.5 1.5 0 0 1-2.122 0l-1.258-1.258a1.5 1.5 0 0 1 0-2.122l1.258-1.258a1.5 1.5 0 0 1 2.122 0Zm-.997-3.04a2.25 2.25 0 0 0-3.182 0l-1.258 1.258a2.25 2.25 0 0 0 0 3.182l1.258 1.258a2.25 2.25 0 0 0 3.182 0l1.258-1.258a2.25 2.25 0 0 0 0-3.182l-1.258-1.258Z")
                            ),
                            "Theme"
                        )),
                        ul().attr("tabindex", "0").class("menu dropdown-content z-[2] p-2 shadow-2xl bg-white/80 backdrop-blur-xl rounded-box w-52 mt-4 border border-slate-200 text-slate-800").child(
                            render_theme_switcher()
                        )
                    ))
                )
            ))
        )
}

fn render_theme_switcher() -> AnyView {
    if let Some(state) = use_context::<sinter_theme_sdk::GlobalState>() {
        let available_themes = state.manager.get_available_themes();
        
        let state_clone = state.clone();
        
        available_themes.into_iter().map(move |name| {
             let s = state_clone.clone();
             let n = name.to_string();
             li().child(
                 a().class("hover:bg-black/5 hover:text-slate-900 rounded-lg transition-colors")
                    .on_click(move || s.switch_theme(&n))
                    .text(name)
             )
        }).collect::<Vec<_>>().into_any()
        
    } else {
         li().child(span().class("text-error").text("Error: Context Missing")).into_any()
    }
}

fn render_footer() -> Element {
    footer().class("footer footer-center p-10 bg-white/40 text-slate-700 backdrop-blur-md border-t border-slate-200/50 mt-auto")
        .child(
            aside().child((
                p().class("font-bold text-lg text-slate-800").child((
                    "Sinter Systems",
                    br(),
                    span().class("font-normal text-sm opacity-60").text("High-performance Content Compilation")
                )),
                p().class("text-xs mt-2 opacity-50").text("Copyright © 2025 - All right reserved")
            ))
        )
}

fn render_pagination(current_page: usize, total_pages: usize, is_archive: bool) -> Element {
    let base_url = if is_archive { "/archives" } else { "/" };
    let prev_url = format!("{}?page={}", base_url, current_page - 1);
    let next_url = format!("{}?page={}", base_url, current_page + 1);

    div().class("flex justify-center items-center gap-4 mt-16 text-slate-700").child((
        if current_page > 1 {
            a().attr("href", prev_url).class("btn btn-circle btn-ghost border-slate-200 hover:bg-slate-100").child(
                svg().class("h-6 w-6").attr("fill", "none").attr("viewBox", "0 0 24 24").attr("stroke", "currentColor").child(
                    path().attr("stroke-linecap", "round").attr("stroke-linejoin", "round").attr("stroke-width", "2").attr("d", "M15 19l-7-7 7-7")
                )
            ).into_any()
        } else {
             button().class("btn btn-circle btn-disabled btn-ghost opacity-20").child(
                svg().class("h-6 w-6").attr("fill", "none").attr("viewBox", "0 0 24 24").attr("stroke", "currentColor").child(
                    path().attr("stroke-linecap", "round").attr("stroke-linejoin", "round").attr("stroke-width", "2").attr("d", "M15 19l-7-7 7-7")
                )
             ).into_any()
        },
        span().class("font-mono opacity-80").text(format!("Page {} of {}", current_page, total_pages)),
        if current_page < total_pages {
            a().attr("href", next_url).class("btn btn-circle btn-ghost border-slate-200 hover:bg-slate-100").child(
                svg().class("h-6 w-6").attr("fill", "none").attr("viewBox", "0 0 24 24").attr("stroke", "currentColor").child(
                    path().attr("stroke-linecap", "round").attr("stroke-linejoin", "round").attr("stroke-width", "2").attr("d", "M9 5l7 7-7 7")
                )
            ).into_any()
        } else {
             button().class("btn btn-circle btn-disabled btn-ghost opacity-20").child(
                svg().class("h-6 w-6").attr("fill", "none").attr("viewBox", "0 0 24 24").attr("stroke", "currentColor").child(
                    path().attr("stroke-linecap", "round").attr("stroke-linejoin", "round").attr("stroke-width", "2").attr("d", "M9 5l7 7-7 7")
                )
             ).into_any()
        }
    ))
}

fn render_post_card(post: SitePostMetadata, is_archive: bool) -> Element {
    let link_base = if is_archive { "/archives/posts/" } else { "/posts/" };
    let slug = post.metadata.slug.clone();
    let link = format!("{}{}", link_base, slug);

    article().class("relative group overflow-hidden rounded-2xl transition-all duration-500 hover:-translate-y-2 mb-12").child((
        div().class("absolute inset-0 bg-white/60 backdrop-blur-md border border-white/50 transition-colors duration-300 group-hover:bg-white/80 shadow-lg"),
        div().class("relative p-8 sm:p-10 text-center z-10").child((
            a().attr("href", link.clone()).class("block group-hover:text-primary transition-colors").child(
                h2().class("text-3xl font-bold mb-4 text-slate-900 tracking-tight group-hover:bg-gradient-to-r group-hover:from-indigo-600 group-hover:to-blue-600 group-hover:bg-clip-text group-hover:text-transparent transition-all")
                    .text(post.metadata.title.clone())
            ),
            div().class("flex flex-wrap justify-center gap-4 text-sm text-slate-600 mb-6 uppercase tracking-wider font-medium").child((
                div().class("flex items-center gap-1").child((
                    svg().class("h-4 w-4").attr("opacity", "0.7").attr("fill", "none").attr("viewBox", "0 0 24 24").attr("stroke", "currentColor").child(
                        path().attr("stroke-linecap", "round").attr("stroke-linejoin", "round").attr("stroke-width", "2").attr("d", "M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z")
                    ),
                    span().text(format_date_slash(&post.metadata.date))
                )),
                div().class("hidden sm:block opacity-50").text("•"),
                div().class("flex items-center gap-2").child(
                    post.metadata.tags.iter().map(|tag| {
                        span().class("px-2 py-0.5 rounded-full bg-slate-200/50 text-slate-700 border border-slate-200").text(tag.clone())
                    }).collect::<Vec<_>>()
                )
            )),
            p().class("text-slate-700 leading-relaxed mb-8 line-clamp-3 max-w-2xl mx-auto").text(post.metadata.summary.clone()),
            div().class("flex justify-center").child(
                a().attr("href", link).class("group/btn relative px-8 py-2 overflow-hidden rounded-full bg-slate-900/5 text-slate-900 transition-all duration-300 hover:bg-slate-900/10 hover:scale-105 hover:shadow-lg border border-slate-900/10").child(
                    span().class("relative z-10 font-medium").text("Read Article")
                )
            )
        ))
    ))
}

fn render_node(node: ContentNode) -> AnyView {
    match node {
        ContentNode::Paragraph { children } => p()
            .class("mb-6 leading-relaxed")
            .child(children.into_iter().map(render_node).collect::<Vec<_>>())
            .into_any(),
        ContentNode::Heading {
            level,
            id,
            classes,
            children,
        } => {
            let content = children.into_iter().map(render_node).collect::<Vec<_>>();
            let id_attr = id.unwrap_or_default();
            let extra_classes = classes.join(" ");

            let el = match level {
                1 => h1().class(format!("text-4xl font-bold mb-8 mt-12 {}", extra_classes)),
                2 => h2().class(format!("text-3xl font-bold mb-6 mt-10 {}", extra_classes)),
                3 => h3().class(format!("text-2xl font-bold mb-4 mt-8 {}", extra_classes)),
                4 => h4().class(format!("text-xl font-bold mb-4 mt-8 {}", extra_classes)),
                5 => h5().class(format!("text-lg font-bold mb-3 mt-6 {}", extra_classes)),
                _ => h6().class(format!("text-base font-bold mb-2 mt-4 {}", extra_classes)),
            };
            
            if !id_attr.is_empty() {
                el.id(id_attr).child(content).into_any()
            } else {
                el.child(content).into_any()
            }
        }
        ContentNode::List { ordered, children } => {
            if ordered {
                ol().class("list-decimal list-inside mb-6 pl-4 space-y-2 marker:text-primary text-slate-700")
                    .child(children.into_iter().map(render_node).collect::<Vec<_>>())
                    .into_any()
            } else {
                ul().class("list-disc list-inside mb-6 pl-4 space-y-2 marker:text-primary text-slate-700")
                    .child(children.into_iter().map(render_node).collect::<Vec<_>>())
                    .into_any()
            }
        }
        ContentNode::ListItem { children } => li()
            .class("text-slate-700")
            .child(children.into_iter().map(render_node).collect::<Vec<_>>())
            .into_any(),
        ContentNode::BlockQuote { children } => blockquote()
            .class("border-l-4 border-primary/50 pl-6 py-4 italic bg-slate-100 rounded-r-lg my-8 text-slate-700")
            .child(children.into_iter().map(render_node).collect::<Vec<_>>())
            .into_any(),
        ContentNode::CodeBlock { lang, code_text } => {
            let lang_label = lang.unwrap_or_else(|| "text".to_string());
            div()
                .class("code-block relative group my-8 rounded-xl overflow-hidden bg-slate-50 text-slate-800 shadow-lg border border-slate-200")
                .child((
                    div().class("flex justify-between items-center px-4 py-2 bg-slate-100 text-xs text-slate-600 select-none border-b border-slate-200").child((
                        span().class("font-mono").text(lang_label),
                        button().class("btn btn-xs btn-ghost gap-1 opacity-0 group-hover:opacity-100 transition-opacity text-slate-600")
                            .attr("aria-label", "Copy code")
                            .child((
                                svg().class("h-3 w-3").attr("fill", "none").attr("viewBox", "0 0 24 24").attr("stroke", "currentColor").child(
                                    path().attr("stroke-linecap", "round").attr("stroke-linejoin", "round").attr("stroke-width", "2").attr("d", "M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z")
                                ),
                                "Copy"
                            ))
                    )),
                    pre().class("p-6 overflow-x-auto font-mono text-sm leading-relaxed !bg-slate-50 !m-0 !rounded-none").child(
                        code().text(code_text)
                    )
                ))
                .into_any()
        }
        ContentNode::Html { value } => {
            let d = div();
            d.dom_element.set_inner_html(&value);
            d.into_any()
        }
        ContentNode::Math { value, display } => {
            let classes = format!(
                "math {} bg-slate-100 px-1.5 py-0.5 rounded border border-slate-200 text-slate-900",
                if display {
                    "block text-center my-6 p-4"
                } else {
                    "inline"
                }
            );
            code().class(classes).text(format!("$ {value} $")).into_any()
        }
        ContentNode::TaskListMarker { checked } => input()
            .attr("type", "checkbox")
            .attr("checked", checked.to_string())
            .attr("disabled", "")
            .class("checkbox checkbox-primary checkbox-xs mr-2 align-middle")
            .into_any(),
        ContentNode::Text { value } => span().class("text-inherit").text(value).into_any(),
        ContentNode::ThematicBreak => hr().class("my-12 border-slate-200").into_any(),
        ContentNode::Emphasis { children } => em()
            .class("italic text-slate-700")
            .child(children.into_iter().map(render_node).collect::<Vec<_>>())
            .into_any(),
        ContentNode::Strong { children } => strong()
            .class("font-bold text-slate-900")
            .child(children.into_iter().map(render_node).collect::<Vec<_>>())
            .into_any(),
        ContentNode::Strikethrough { children } => s()
            .class("line-through opacity-60")
            .child(children.into_iter().map(render_node).collect::<Vec<_>>())
            .into_any(),
        ContentNode::Link {
            url,
            title,
            children,
        } => a()
            .attr("href", url)
            .attr("title", title.unwrap_or_default())
            .class("link link-primary hover:text-primary-focus transition-colors decoration-2 decoration-primary/30 hover:decoration-primary")
            .child(children.into_iter().map(render_node).collect::<Vec<_>>())
            .into_any(),
        ContentNode::Image { url, title, alt } => figure()
            .class("my-10")
            .child((
                img()
                    .attr("src", url)
                    .attr("alt", alt)
                    .attr("title", title.clone().unwrap_or_default())
                    .class("rounded-xl shadow-lg mx-auto max-w-full border border-slate-200")
                    .attr("loading", "lazy"),
               if let Some(t) = title {
                   figcaption().class("text-center text-sm mt-3 opacity-60 italic").text(t).into_any()
               } else {
                   div().style("display: none").into_any()
               }
            ))
            .into_any(),
        ContentNode::Table { children } => div()
            .class("overflow-x-auto my-10 rounded-xl border border-slate-200 bg-slate-50")
            .child(
                table()
                    .class("table table-zebra w-full text-left text-slate-700")
                    .child(children.into_iter().map(render_node).collect::<Vec<_>>())
            )
            .into_any(),
        ContentNode::TableHead { children } => thead()
            .class("bg-slate-200 text-slate-900 font-bold")
            .child(children.into_iter().map(render_node).collect::<Vec<_>>())
            .into_any(),
        ContentNode::TableBody { children } => tbody()
            .child(children.into_iter().map(render_node).collect::<Vec<_>>())
            .into_any(),
        ContentNode::TableRow { children } => tr()
            .class("border-b border-slate-200 hover:bg-slate-100 transition-colors")
            .child(children.into_iter().map(render_node).collect::<Vec<_>>())
            .into_any(),
        ContentNode::TableCell { children } => td()
            .class("px-6 py-4 whitespace-pre-wrap")
            .child(children.into_iter().map(render_node).collect::<Vec<_>>())
            .into_any(),
    }
}

fn format_date_slash(date: &sinter_core::LiteDate) -> String {
    format!("{}/{:02}/{:02}", date.year, date.month, date.day)
}

fn format_date_long(date: &sinter_core::LiteDate) -> String {
    let month = match date.month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "",
    };
    format!("{} {}, {}", month, date.day, date.year)
}
