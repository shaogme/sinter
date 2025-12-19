use leptos::prelude::*;
use sinter_core::{ContentNode, Post, SiteMetaData};
use sinter_theme_sdk::Theme;

#[derive(Clone, Debug)]
pub struct DefaultTheme;

impl Theme for DefaultTheme {
    fn render_layout(&self, children: Children, site_data: Signal<Option<SiteMetaData>>) -> AnyView {
        let site_title = move || site_data.get().map(|d| d.title).unwrap_or_else(|| "Sinter".to_string());

        view! {
            <div class="flex flex-col min-h-screen font-sans text-base-content relative">
                // --- Aurora Background ---
                <div class="aurora-bg">
                    <div class="blob"></div>
                    <div class="blob"></div>
                    <div class="blob"></div>
                </div>
                <div class="overlay"></div>
                // --- SVG Filter 注入 (完全对应 index.html 参数) ---
                <svg 
                    xmlns="http://www.w3.org/2000/svg"
                    style="position: absolute; width: 0; height: 0; overflow: hidden; pointer-events: none;"
                    aria-hidden="true"
                >
                    <defs>
                        <filter id="glass-distortion" x="0%" y="0%" width="100%" height="100%" filterUnits="objectBoundingBox">
                            // 1. 湍流噪点: baseFrequency="0.01 0.01", seed="5" (对应参考文件)
                            <feTurbulence type="fractalNoise" baseFrequency="0.01 0.01" numOctaves="1" seed="5" result="turbulence" />
                            
                            // 2. 颜色通道映射
                            <feComponentTransfer in="turbulence" result="mapped">
                                <feFuncR type="gamma" amplitude="1" exponent="10" offset="0.5" />
                                <feFuncG type="gamma" amplitude="0" exponent="1" offset="0" />
                                <feFuncB type="gamma" amplitude="0" exponent="1" offset="0.5" />
                            </feComponentTransfer>

                            // 3. 高斯模糊
                            <feGaussianBlur in="turbulence" stdDeviation="3" result="softMap" />

                            // 4. 镜面光照: surfaceScale="5", specularExponent="100"
                            <feSpecularLighting in="softMap" surfaceScale="5" specularConstant="1" specularExponent="100" attr:lighting-color="white" result="specLight">
                                <fePointLight x="-200" y="-200" z="300" />
                            </feSpecularLighting>

                            // 5. 合成光照
                            <feComposite in="specLight" operator="arithmetic" k1="0" k2="1" k3="1" k4="0" result="litImage" />

                            // 6. 图像置换: scale="150" (对应参考文件)
                            <feDisplacementMap in="SourceGraphic" in2="softMap" scale="150" xChannelSelector="R" yChannelSelector="G" result="distorted" />
                            
                            // 7. 最终合成 (将光照叠加在扭曲图像上)
                            // 注意：这里需要再次合成，确保光照效果覆盖在最终图像上
                            <feComposite in="specLight" in2="distorted" operator="in" result="final" />
                        </filter>
                    </defs>
                </svg>

                // Navbar
                <nav class="navbar fixed top-0 z-50 transition-all duration-300 hover:bg-white/10 hover:shadow-lg text-primary-content has-[.scrolled]:bg-black/20 backdrop-blur-md border-b border-white/5">
                    <div class="container mx-auto px-4 flex items-center">
                        <div class="flex-1">
                            <a href="/" class="btn btn-ghost text-2xl font-black tracking-tighter hover:scale-105 transition-transform text-white drop-shadow-md">
                                {site_title}
                            </a>
                        </div>
                        <div class="flex-none hidden md:block">
                            <ul class="menu menu-horizontal px-1 font-medium text-white/90">
                                <li><a href="/" class="hover:bg-white/10 hover:text-white transition-all rounded-lg">"Home"</a></li>
                                <li><a href="/archives" class="hover:bg-white/10 hover:text-white transition-all rounded-lg">"Archives"</a></li>
                            </ul>
                        </div>
                        <div class="flex-none">
                            <div class="dropdown dropdown-end">
                                <div tabindex="0" role="button" class="btn btn-ghost hover:bg-white/10 text-white rounded-btn gap-2">
                                     <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                                      <path stroke-linecap="round" stroke-linejoin="round" d="M4.098 19.902a3.75 3.75 0 0 0 5.304 0l4.56-4.56a3.75 3.75 0 0 0-5.304-5.304l-4.56 4.56a3.75 3.75 0 0 0 0 5.304Zm1.706-9.191a1.5 1.5 0 0 1 2.122 0l2.122 2.121a1.5 1.5 0 0 1 0 2.122l-2.122 2.122a1.5 1.5 0 0 1-2.122 0l-2.122-2.122a1.5 1.5 0 0 1 0-2.122l2.122-2.122Zm9.706-5.464 1.258 1.258a1.5 1.5 0 0 1 0 2.122l-1.258 1.258a1.5 1.5 0 0 1-2.122 0l-1.258-1.258a1.5 1.5 0 0 1 0-2.122l1.258-1.258a1.5 1.5 0 0 1 2.122 0Zm-.997-3.04a2.25 2.25 0 0 0-3.182 0l-1.258 1.258a2.25 2.25 0 0 0 0 3.182l1.258 1.258a2.25 2.25 0 0 0 3.182 0l1.258-1.258a2.25 2.25 0 0 0 0-3.182l-1.258-1.258Z" />
                                    </svg>
                                    "Theme"
                                </div>
                                <ul tabindex="0" class="menu dropdown-content z-[2] p-2 shadow-2xl bg-black/50 backdrop-blur-xl rounded-box w-52 mt-4 border border-white/10 text-white">
                                    {
                                    let state_opt = use_context::<sinter_theme_sdk::GlobalState>();
                                    if let Some(state) = state_opt {
                                        view! {
                                            <>{
                                                state.manager.get_available_themes().into_iter().map(|name| {
                                                    let state = state.clone();
                                                    view! {
                                                        <li>
                                                            <a 
                                                                class="hover:bg-white/10 hover:text-white rounded-lg transition-colors"
                                                                on:click=move |_| state.switch_theme(name)
                                                            >
                                                                {name}
                                                            </a>
                                                        </li>
                                                    }
                                                }).collect_view()
                                            }</>
                                        }.into_any()
                                    } else {
                                        view! { <li><span class="text-error">"Error: Context Missing"</span></li> }.into_any()
                                    }
                                    }
                                </ul>
                            </div>
                        </div>
                    </div>
                </nav>

                // Main Content
                <main class="flex-grow w-full">
                    {children()}
                </main>

                // Footer
                <footer class="footer footer-center p-10 bg-black/20 text-white/70 backdrop-blur-md border-t border-white/5 mt-auto">
                    <aside>
                        <p class="font-bold text-lg text-white">
                            "Sinter Systems"
                            <br/>
                            <span class="font-normal text-sm opacity-60">"High-performance Content Compilation"</span>
                        </p>
                        <p class="text-xs mt-2 opacity-50">"Copyright © 2025 - All right reserved"</p>
                    </aside>
                </footer>
            </div>
        }.into_any()
    }

    fn render_home(&self) -> AnyView {
        // Use hooks to access data
        let site_meta_r = sinter_theme_sdk::use_site_meta();
        let page_data_r = sinter_theme_sdk::use_page_data();
        let current_page_s = sinter_theme_sdk::use_current_page();
        
        let theme = self.clone();
        let theme_fallback = theme.clone();

        view! {
            <Suspense fallback=move || theme_fallback.render_loading()>
                {move || {
                    let site_meta_res = site_meta_r.and_then(|r| r.get()).and_then(|r| r.ok());
                    let page_data_res = page_data_r.clone().and_then(|r| r.get().and_then(|res| res.ok()));

                    match (site_meta_res, page_data_res) {
                        (Some(site_meta), Some(page_data)) => {
                            let posts = page_data.posts;
                            let title = site_meta.title.clone();
                            let subtitle = site_meta.subtitle.clone();
                            let description = site_meta.description.clone();
                            let total_pages = site_meta.total_pages;

                            let search = current_page_s.get();

                            view! {
                                <div class="flex flex-col w-full">
                                    // Hero Section
                                    <div class="hero min-h-screen relative">
                                        <div class="hero-content text-center text-neutral-content z-10 w-full">
                                            <div class="w-full max-w-7xl mx-auto flex flex-col items-center animate-fade-in-up">
                                                
                                                // --- Liquid Glass Component ---
                                                <div class="mb-12">
                                                    <div class="liquidGlass-wrapper">
                                                        <div class="liquidGlass-effect"></div>
                                                        <div class="liquidGlass-tint"></div>
                                                        <div class="liquidGlass-shine"></div>
                                                        
                                                        <div class="liquidGlass-text">
                                                            <h1 class="text-6xl md:text-8xl lg:text-9xl font-black tracking-tighter leading-none">
                                                                {title}
                                                            </h1>
                                                        </div>
                                                    </div>
                                                </div>

                                                <div class="text-center text-white space-y-6 max-w-2xl mx-auto px-4">
                                                    <h2 class="text-2xl md:text-4xl font-bold opacity-90 drop-shadow-lg">
                                                        {subtitle}
                                                    </h2>
                                                    <p class="text-lg md:text-2xl font-medium opacity-80 drop-shadow-md">
                                                        {description}
                                                    </p>
                                                </div>

                                                <div class="absolute bottom-10 left-1/2 -translate-x-1/2 animate-bounce">
                                                    <svg class="h-10 w-10 text-white/80" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3" />
                                                    </svg>
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                    // Posts Grid
                                    <div class="py-20 px-4 min-h-[50vh]">
                                        <div class="container mx-auto max-w-5xl space-y-12">
                                            <For
                                                each=move || posts.clone()
                                                key=|post| post.metadata.id.clone()
                                                children=|post| view! {
                                                    <article class="relative group overflow-hidden rounded-2xl transition-all duration-500 hover:-translate-y-2">
                                                        <div class="absolute inset-0 bg-white/5 backdrop-blur-md border border-white/10 transition-colors duration-300 group-hover:bg-white/10 shadow-lg"></div>
                                                        
                                                        <div class="relative p-8 sm:p-10 text-center z-10">
                                                            <a href=format!("/posts/{}", post.metadata.slug) class="block group-hover:text-primary-content transition-colors">
                                                                <h2 class="text-3xl font-bold mb-4 text-white tracking-tight group-hover:bg-gradient-to-r group-hover:from-white group-hover:to-white/70 group-hover:bg-clip-text group-hover:text-transparent transition-all">
                                                                    {post.metadata.title.clone()}
                                                                </h2>
                                                            </a>
                                                            
                                                            <div class="flex flex-wrap justify-center gap-4 text-sm text-gray-300 mb-6 uppercase tracking-wider font-medium">
                                                                <div class="flex items-center gap-1">
                                                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" opacity="0.7" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                                                                    </svg>
                                                                    <span>{format_date_slash(&post.metadata.date)}</span>
                                                                </div>
                                                                <div class="hidden sm:block opacity-50">"•"</div>
                                                            <div class="flex items-center gap-2">
                                                                    {post.metadata.tags.iter().map(|tag: &String| view! {
                                                                        <span class="px-2 py-0.5 rounded-full bg-white/10 text-white/90 border border-white/5">{tag.clone()}</span>
                                                                    }).collect::<Vec<_>>()}
                                                                </div>
                                                            </div>

                                                            <p class="text-gray-300/80 leading-relaxed mb-8 line-clamp-3 max-w-2xl mx-auto">
                                                                {post.metadata.summary.clone()}
                                                            </p>

                                                            <div class="flex justify-center">
                                                                <a href=format!("/posts/{}", post.metadata.slug) class="group/btn relative px-8 py-2 overflow-hidden rounded-full bg-white/10 text-white transition-all duration-300 hover:bg-white/20 hover:scale-105 hover:shadow-[0_0_20px_rgba(255,255,255,0.3)] border border-white/10">
                                                                    <span class="relative z-10 font-medium">"Read Article"</span>
                                                                </a>
                                                            </div>
                                                        </div>
                                                    </article>
                                                }
                                            />

                                            // Pagination Controls
                                            <div class="flex justify-center items-center gap-4 mt-16 text-white">
                                                {if search > 1 {
                                                    view! {
                                                        <a href=format!("/?page={}", search - 1) class="btn btn-circle btn-ghost border-white/10 hover:bg-white/10">
                                                            <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" /></svg>
                                                        </a>
                                                    }.into_any()
                                                } else {
                                                    view! { <button class="btn btn-circle btn-disabled btn-ghost opacity-20"><svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" /></svg></button> }.into_any()
                                                }}
                                                
                                                <span class="font-mono opacity-80">{format!("Page {} of {}", search, total_pages)}</span>

                                                {if search < total_pages {
                                                    view! {
                                                        <a href=format!("/?page={}", search + 1) class="btn btn-circle btn-ghost border-white/10 hover:bg-white/10">
                                                            <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" /></svg>
                                                        </a>
                                                    }.into_any()
                                                } else {
                                                    view! { <button class="btn btn-circle btn-disabled btn-ghost opacity-20"><svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" /></svg></button> }.into_any()
                                                }}
                                            </div>

                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        },
                        // Either meta or page data missing/error
                        _ => theme.render_loading() 
                    }
                }}
            </Suspense>
        }.into_any()
    }

    fn render_archive(&self) -> AnyView {
        // Use hooks to access data
        let site_meta_r = sinter_theme_sdk::use_site_meta();
        let page_data_r = sinter_theme_sdk::use_page_data();
        let current_page_s = sinter_theme_sdk::use_current_page();
        
        let theme = self.clone();
        let theme_fallback = theme.clone();

        view! {
            <Suspense fallback=move || theme_fallback.render_loading()>
                {move || {
                    let site_meta_res = site_meta_r.and_then(|r| r.get()).and_then(|r| r.ok());
                    let page_data_res = page_data_r.clone().and_then(|r| r.get().and_then(|res| res.ok()));

                    match (site_meta_res, page_data_res) {
                        (Some(site_meta), Some(page_data)) => {
                            let posts = page_data.posts;
                            let title = site_meta.title.clone();
                            let subtitle = site_meta.subtitle.clone();
                            let description = site_meta.description.clone();
                            let total_pages = site_meta.total_pages;

                            let search = current_page_s.get();

                            view! {
                                <div class="flex flex-col w-full">
                                    // Hero Section
                                    <div class="hero min-h-screen relative">
                                        <div class="hero-content text-center text-neutral-content z-10 w-full">
                                            <div class="w-full max-w-7xl mx-auto flex flex-col items-center animate-fade-in-up">
                                                
                                                // --- Liquid Glass Component ---
                                                <div class="mb-12">
                                                    <div class="liquidGlass-wrapper">
                                                        <div class="liquidGlass-effect"></div>
                                                        <div class="liquidGlass-tint"></div>
                                                        <div class="liquidGlass-shine"></div>
                                                        
                                                        <div class="liquidGlass-text">
                                                            <h1 class="text-6xl md:text-8xl lg:text-9xl font-black tracking-tighter leading-none">
                                                                {title}
                                                            </h1>
                                                        </div>
                                                    </div>
                                                </div>

                                                <div class="text-center text-white space-y-6 max-w-2xl mx-auto px-4">
                                                    <h2 class="text-2xl md:text-4xl font-bold opacity-90 drop-shadow-lg">
                                                        {subtitle}
                                                    </h2>
                                                    <p class="text-lg md:text-2xl font-medium opacity-80 drop-shadow-md">
                                                        {description}
                                                    </p>
                                                </div>

                                                <div class="absolute bottom-10 left-1/2 -translate-x-1/2 animate-bounce">
                                                    <svg class="h-10 w-10 text-white/80" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 14l-7 7m0 0l-7-7m7 7V3" />
                                                    </svg>
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                    // Posts Grid
                                    <div class="py-20 px-4 min-h-[50vh]">
                                        <div class="container mx-auto max-w-5xl space-y-12">
                                            <For
                                                each=move || posts.clone()
                                                key=|post| post.metadata.id.clone()
                                                children=|post| view! {
                                                    <article class="relative group overflow-hidden rounded-2xl transition-all duration-500 hover:-translate-y-2">
                                                        <div class="absolute inset-0 bg-white/5 backdrop-blur-md border border-white/10 transition-colors duration-300 group-hover:bg-white/10 shadow-lg"></div>
                                                        
                                                        <div class="relative p-8 sm:p-10 text-center z-10">
                                                            <a href=format!("/archives/posts/{}", post.metadata.slug) class="block group-hover:text-primary-content transition-colors">
                                                                <h2 class="text-3xl font-bold mb-4 text-white tracking-tight group-hover:bg-gradient-to-r group-hover:from-white group-hover:to-white/70 group-hover:bg-clip-text group-hover:text-transparent transition-all">
                                                                    {post.metadata.title.clone()}
                                                                </h2>
                                                            </a>
                                                            
                                                            <div class="flex flex-wrap justify-center gap-4 text-sm text-gray-300 mb-6 uppercase tracking-wider font-medium">
                                                                <div class="flex items-center gap-1">
                                                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" opacity="0.7" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                                                                    </svg>
                                                                    <span>{format_date_slash(&post.metadata.date)}</span>
                                                                </div>
                                                                <div class="hidden sm:block opacity-50">"•"</div>
                                                            <div class="flex items-center gap-2">
                                                                    {post.metadata.tags.iter().map(|tag: &String| view! {
                                                                        <span class="px-2 py-0.5 rounded-full bg-white/10 text-white/90 border border-white/5">{tag.clone()}</span>
                                                                    }).collect::<Vec<_>>()}
                                                                </div>
                                                            </div>

                                                            <p class="text-gray-300/80 leading-relaxed mb-8 line-clamp-3 max-w-2xl mx-auto">
                                                                {post.metadata.summary.clone()}
                                                            </p>

                                                            <div class="flex justify-center">
                                                                <a href=format!("/archives/posts/{}", post.metadata.slug) class="group/btn relative px-8 py-2 overflow-hidden rounded-full bg-white/10 text-white transition-all duration-300 hover:bg-white/20 hover:scale-105 hover:shadow-[0_0_20px_rgba(255,255,255,0.3)] border border-white/10">
                                                                    <span class="relative z-10 font-medium">"Read Article"</span>
                                                                </a>
                                                            </div>
                                                        </div>
                                                    </article>
                                                }
                                            />

                                            // Pagination Controls
                                            <div class="flex justify-center items-center gap-4 mt-16 text-white">
                                                {if search > 1 {
                                                    view! {
                                                        <a href=format!("/archives?page={}", search - 1) class="btn btn-circle btn-ghost border-white/10 hover:bg-white/10">
                                                            <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" /></svg>
                                                        </a>
                                                    }.into_any()
                                                } else {
                                                    view! { <button class="btn btn-circle btn-disabled btn-ghost opacity-20"><svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" /></svg></button> }.into_any()
                                                }}
                                                
                                                <span class="font-mono opacity-80">{format!("Page {} of {}", search, total_pages)}</span>

                                                {if search < total_pages {
                                                    view! {
                                                        <a href=format!("/archives?page={}", search + 1) class="btn btn-circle btn-ghost border-white/10 hover:bg-white/10">
                                                            <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" /></svg>
                                                        </a>
                                                    }.into_any()
                                                } else {
                                                    view! { <button class="btn btn-circle btn-disabled btn-ghost opacity-20"><svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" /></svg></button> }.into_any()
                                                }}
                                            </div>

                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        },
                        // Either meta or page data missing/error
                        _ => theme.render_loading() 
                    }
                }}
            </Suspense>
        }.into_any()
    }

    
    fn render_post(&self, post: Post) -> AnyView {
        view! {
            <div class="pt-24 lg:pt-32 pb-20 px-4">
                <article class="max-w-4xl mx-auto animate-fade-in relative">
                    // Glass Container
                    <div class="absolute inset-0 -mx-4 sm:-mx-12 bg-black/20 backdrop-blur-xl rounded-[2.5rem] border border-white/5 shadow-2xl z-0"></div>
                    
                    // Content Wrapper
                    <div class="relative z-10 p-4 sm:p-12">
                        <header class="text-center mb-16 space-y-6">
                            <h1 class="text-4xl md:text-5xl lg:text-6xl font-black text-white leading-tight drop-shadow-lg">
                                {post.metadata.title.clone()}
                            </h1>

                            <div class="flex flex-wrap items-center justify-center gap-4 text-sm font-medium text-gray-300">
                                <time class="px-4 py-1.5 rounded-full bg-white/5 border border-white/5 backdrop-blur-sm">
                                    {format_date_long(&post.metadata.date)}
                                </time>
                                <div class="flex gap-2">
                                    {post.metadata.tags.iter().map(|tag| view! {
                                        <span class="px-3 py-1 rounded-full bg-primary/20 text-primary-content border border-primary/20 backdrop-blur-sm uppercase tracking-wider text-xs">{tag.clone()}</span>
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                        </header>

                        <div class="prose prose-lg prose-invert mx-auto max-w-none prose-headings:text-white prose-p:text-gray-200 prose-a:text-blue-300 prose-blockquote:border-l-primary prose-code:text-primary-content">
                            {post.content_ast.iter().map(|node| view! { <NodeRenderer node=node.clone() /> }).collect_view()}
                        </div>

                        <div class="mt-20 pt-10 border-t border-white/10 text-center">
                            <a href="/" class="btn btn-ghost hover:bg-white/10 text-white gap-3 rounded-full px-8">
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" /></svg>
                                "Back to Home"
                            </a>
                        </div>
                    </div>
                </article>
            </div>
        }.into_any()
    }

    fn render_post_loading(&self) -> AnyView {
        view! {
            <div class="flex justify-center items-center min-h-screen pt-20">
                <span class="loading loading-spinner loading-lg text-primary"></span>
            </div>
        }
        .into_any()
    }

    fn render_loading(&self) -> AnyView {
        view! {
            <div class="flex justify-center items-center h-full w-full min-h-[50vh]">
                <span class="loading loading-dots loading-lg text-secondary"></span>
            </div>
        }
        .into_any()
    }

    fn render_post_not_found(&self) -> AnyView {
        view! {
            <div class="hero min-h-screen pt-16">
                <div class="hero-content text-center">
                    <div class="max-w-md space-y-8">
                        <h1 class="text-9xl font-black text-white/10">"404"</h1>
                        <h2 class="text-4xl font-bold text-white">"Page Not Found"</h2>
                        <p class="text-lg text-white/60">"The content you're looking for seems to have been moved or deleted."</p>
                        <a href="/" class="btn btn-primary btn-lg min-w-[200px]">"Return Home"</a>
                    </div>
                </div>
            </div>
        }.into_any()
    }

    fn render_error(&self, message: String) -> AnyView {
        view! {
            <div class="flex justify-center items-center h-full min-h-[50vh] p-4">
                <div class="alert alert-error shadow-lg rounded-xl max-w-lg">
                    <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current flex-shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                    <div>
                        <h3 class="font-bold">"Error loading content"</h3>
                        <div class="text-xs break-words mt-1">{message}</div>
                    </div>
                </div>
            </div>
        }.into_any()
    }
}

// Internal helper for DefaultTheme
#[component]
fn NodeRenderer(node: ContentNode) -> impl IntoView {
    match node {
        ContentNode::Paragraph { children } => view! {
            <p class="mb-6 leading-relaxed">
                {children.into_iter().map(|c| view! { <NodeRenderer node=c /> }).collect_view()}
            </p>
        }.into_any(),
        ContentNode::Heading { level, id, classes, children } => {
            let content = view! { {children.into_iter().map(|c| view! { <NodeRenderer node=c /> }).collect_view()} };
            let id_attr = id.unwrap_or_default();
            let extra_classes = classes.join(" ");

            match level {
                1 => view! { <h1 id=id_attr class=format!("text-4xl font-bold mb-8 mt-12 {}", extra_classes)>{content}</h1> }.into_any(),
                2 => view! { <h2 id=id_attr class=format!("text-3xl font-bold mb-6 mt-10 {}", extra_classes)>{content}</h2> }.into_any(),
                3 => view! { <h3 id=id_attr class=format!("text-2xl font-bold mb-4 mt-8 {}", extra_classes)>{content}</h3> }.into_any(),
                4 => view! { <h4 id=id_attr class=format!("text-xl font-bold mb-4 mt-8 {}", extra_classes)>{content}</h4> }.into_any(),
                5 => view! { <h5 id=id_attr class=format!("text-lg font-bold mb-3 mt-6 {}", extra_classes)>{content}</h5> }.into_any(),
                _ => view! { <h6 id=id_attr class=format!("text-base font-bold mb-2 mt-4 {}", extra_classes)>{content}</h6> }.into_any(),
            }
        },
        ContentNode::List { ordered, children } => {
            if ordered {
                view! { <ol class="list-decimal list-inside mb-6 pl-4 space-y-2 marker:text-primary">{children.into_iter().map(|c| view! { <NodeRenderer node=c /> }).collect_view()}</ol> }.into_any()
            } else {
                view! { <ul class="list-disc list-inside mb-6 pl-4 space-y-2 marker:text-primary">{children.into_iter().map(|c| view! { <NodeRenderer node=c /> }).collect_view()}</ul> }.into_any()
            }
        },
        ContentNode::ListItem { children } => view! {
             <li>{children.into_iter().map(|c| view! { <NodeRenderer node=c /> }).collect_view()}</li>
        }.into_any(),
        ContentNode::BlockQuote { children } => view! {
            <blockquote class="border-l-4 border-primary/50 pl-6 py-4 italic bg-white/5 rounded-r-lg my-8 text-gray-300">
                {children.into_iter().map(|c| view! { <NodeRenderer node=c /> }).collect_view()}
            </blockquote>
        }.into_any(),
        ContentNode::CodeBlock { lang, code } => {
            let lang_label = lang.unwrap_or_else(|| "text".to_string());
            view! {
                <div class="code-block relative group my-8 rounded-xl overflow-hidden bg-black/50 backdrop-blur-md text-gray-200 shadow-2xl border border-white/10">
                    <div class="flex justify-between items-center px-4 py-2 bg-white/5 text-xs text-gray-400 select-none border-b border-white/5">
                        <span class="font-mono">{lang_label}</span>
                        <button class="btn btn-xs btn-ghost gap-1 opacity-0 group-hover:opacity-100 transition-opacity text-gray-300"
                                aria-label="Copy code">
                             <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                             </svg>
                             "Copy"
                        </button>
                    </div>
                    <pre class="p-6 overflow-x-auto font-mono text-sm leading-relaxed !bg-white/5 !m-0 !rounded-none">
                        <code>{code}</code>
                    </pre>
                </div>
            }.into_any()
        },
        ContentNode::Html { value } => view! { <div inner_html=value></div> }.into_any(),
        ContentNode::Math { value, display } => {
             view! { <code class=format!("math {} bg-white/5 px-1.5 py-0.5 rounded border border-white/10 text-gray-200", if display { "block text-center my-6 p-4" } else { "inline" })>{ "$ " }{value}{ " $" }</code> }.into_any()
        },
        ContentNode::TaskListMarker { checked } => view! {
             <input type="checkbox" checked=checked disabled class="checkbox checkbox-primary checkbox-xs mr-2 align-middle" />
        }.into_any(),
        ContentNode::Text { value } => view! { <span class="text-inherit">{value}</span> }.into_any(),
        ContentNode::ThematicBreak => view! { <hr class="my-12 border-white/10" /> }.into_any(),
        ContentNode::Emphasis { children } => view! { <em class="italic text-gray-300">{children.into_iter().map(|c| view! { <NodeRenderer node=c /> }).collect_view()}</em> }.into_any(),
        ContentNode::Strong { children } => view! { <strong class="font-bold text-white">{children.into_iter().map(|c| view! { <NodeRenderer node=c /> }).collect_view()}</strong> }.into_any(),
        ContentNode::Strikethrough { children } => view! { <s class="line-through opacity-60">{children.into_iter().map(|c| view! { <NodeRenderer node=c /> }).collect_view()}</s> }.into_any(),
        ContentNode::Link { url, title, children } => view! {
            <a href=url title=title.unwrap_or_default() class="link link-primary hover:text-primary-focus transition-colors decoration-2 decoration-primary/30 hover:decoration-primary">
                {children.into_iter().map(|c| view! { <NodeRenderer node=c /> }).collect_view()}
            </a>
        }.into_any(),
        ContentNode::Image { url, title, alt } => view! {
            <figure class="my-10">
                <img src=url alt=alt title=title.clone().unwrap_or_default() class="rounded-xl shadow-2xl mx-auto max-w-full border border-white/5" loading="lazy" />
                {move || title.as_ref().map(|t| view! { <figcaption class="text-center text-sm mt-3 opacity-60 italic">{t.clone()}</figcaption> })}
            </figure>
        }.into_any(),
        ContentNode::Table { children } => view! {
            <div class="overflow-x-auto my-10 rounded-xl border border-white/10 bg-white/5">
                <table class="table table-zebra w-full text-left text-gray-300">
                    {children.into_iter().map(|c| view! { <NodeRenderer node=c /> }).collect_view()}
                </table>
            </div>
        }.into_any(),
        ContentNode::TableHead { children } => view! { <thead class="bg-white/10 text-white font-bold">{children.into_iter().map(|c| view! { <NodeRenderer node=c /> }).collect_view()}</thead> }.into_any(),
        ContentNode::TableBody { children } => view! { <tbody>{children.into_iter().map(|c| view! { <NodeRenderer node=c /> }).collect_view()}</tbody> }.into_any(),
        ContentNode::TableRow { children } => view! { <tr class="border-b border-white/5 hover:bg-white/5 transition-colors">{children.into_iter().map(|c| view! { <NodeRenderer node=c /> }).collect_view()}</tr> }.into_any(),
        ContentNode::TableCell { children } => view! { <td class="px-6 py-4 whitespace-pre-wrap">{children.into_iter().map(|c| view! { <NodeRenderer node=c /> }).collect_view()}</td> }.into_any(),
    }
}

fn format_date_slash(date: &sinter_core::LiteDate) -> String {
    format!("{}/{:02}/{:02}", date.year, date.month, date.day)
}

fn format_date_long(date: &sinter_core::LiteDate) -> String {
    let month = match date.month {
        1 => "January", 2 => "February", 3 => "March", 4 => "April",
        5 => "May", 6 => "June", 7 => "July", 8 => "August",
        9 => "September", 10 => "October", 11 => "November", 12 => "December",
        _ => "",
    };
    format!("{} {}, {}", month, date.day, date.year)
}