use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use chrono::{Datelike, Utc};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use utoipa::{IntoParams, OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_languages,
        get_language_info
    ),
    components(
        schemas(Language, LanguageDetail, DateQuery)
    ),
    tags(
        (name = "rust-tiobe", description = "Rust TIOBE Index API")
    )
)]
struct ApiDoc;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
struct Language {
    rank: i32,
    prev_rank: i32,
    name: String,
    rating: String,
    change: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct LanguageDetail {
    name: String,
    rank: i32,
    rating: String,
    description: String,
    use_cases: Vec<String>,
    frameworks: Vec<String>,
}

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
struct DateQuery {
    year: Option<i32>,
    month: Option<i32>,
}

async fn fetch_tiobe_data(year: Option<i32>, month: Option<i32>) -> Result<Vec<Language>, String> {
    let client = reqwest::Client::new();
    
    // 构建 URL，支持历史数据
    let url = match (year, month) {
        (Some(y), Some(m)) => {
            // 验证不是未来时间
            let now = Utc::now();
            let current_year = now.year();
            let current_month = now.month() as i32;
            
            if y > current_year || (y == current_year && m > current_month) {
                return Err("不能查询未来时间".to_string());
            }
            format!("https://www.tiobe.com/tiobe-index/?page=index&year={}&month={}", y, m)
        }
        _ => "https://www.tiobe.com/tiobe-index/".to_string(),
    };

    let resp = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let body = resp.text().await.map_err(|e| e.to_string())?;
    let document = Html::parse_document(&body);

    let table_selector = Selector::parse("table#top20 tbody tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();

    let mut languages = Vec::new();

    for row in document.select(&table_selector) {
        let cells: Vec<_> = row.select(&td_selector).collect();
        if cells.len() >= 5 {
            let rank: i32 = cells[0].text().collect::<String>().trim().parse().unwrap_or(0);
            let prev_rank: i32 = cells[1].text().collect::<String>().trim().parse().unwrap_or(0);
            let name = cells[3].text().collect::<String>().trim().to_string();
            let rating = cells[4].text().collect::<String>().trim().to_string();
            let change = if cells.len() > 5 {
                cells[5].text().collect::<String>().trim().to_string()
            } else {
                "N/A".to_string()
            };

            if !name.is_empty() && rank > 0 {
                languages.push(Language {
                    rank,
                    prev_rank,
                    name,
                    rating,
                    change,
                });
            }
        }
    }

    if languages.is_empty() {
        languages = get_fallback_data();
    }

    Ok(languages)
}

fn get_fallback_data() -> Vec<Language> {
    vec![
        Language { rank: 1, prev_rank: 1, name: "Python".to_string(), rating: "23.64%".to_string(), change: "-0.21%".to_string() },
        Language { rank: 2, prev_rank: 4, name: "C".to_string(), rating: "10.11%".to_string(), change: "+1.01%".to_string() },
        Language { rank: 3, prev_rank: 2, name: "C++".to_string(), rating: "8.95%".to_string(), change: "-1.87%".to_string() },
        Language { rank: 4, prev_rank: 3, name: "Java".to_string(), rating: "8.70%".to_string(), change: "-1.02%".to_string() },
        Language { rank: 5, prev_rank: 5, name: "C#".to_string(), rating: "7.26%".to_string(), change: "+2.39%".to_string() },
        Language { rank: 6, prev_rank: 6, name: "JavaScript".to_string(), rating: "2.96%".to_string(), change: "-1.66%".to_string() },
        Language { rank: 7, prev_rank: 9, name: "Visual Basic".to_string(), rating: "2.81%".to_string(), change: "+0.85%".to_string() },
        Language { rank: 8, prev_rank: 8, name: "SQL".to_string(), rating: "2.10%".to_string(), change: "+0.11%".to_string() },
        Language { rank: 9, prev_rank: 26, name: "Perl".to_string(), rating: "1.97%".to_string(), change: "+1.33%".to_string() },
        Language { rank: 10, prev_rank: 16, name: "R".to_string(), rating: "1.96%".to_string(), change: "+0.91%".to_string() },
        Language { rank: 11, prev_rank: 11, name: "Delphi/Object Pascal".to_string(), rating: "1.91%".to_string(), change: "+0.48%".to_string() },
        Language { rank: 12, prev_rank: 10, name: "Fortran".to_string(), rating: "1.60%".to_string(), change: "-0.18%".to_string() },
        Language { rank: 13, prev_rank: 15, name: "MATLAB".to_string(), rating: "1.52%".to_string(), change: "+0.43%".to_string() },
        Language { rank: 14, prev_rank: 24, name: "Ada".to_string(), rating: "1.49%".to_string(), change: "+0.77%".to_string() },
        Language { rank: 15, prev_rank: 7, name: "Go".to_string(), rating: "1.37%".to_string(), change: "-0.80%".to_string() },
        Language { rank: 16, prev_rank: 12, name: "PHP".to_string(), rating: "1.36%".to_string(), change: "-0.03%".to_string() },
        Language { rank: 17, prev_rank: 14, name: "Rust".to_string(), rating: "1.30%".to_string(), change: "+0.01%".to_string() },
        Language { rank: 18, prev_rank: 13, name: "Scratch".to_string(), rating: "1.11%".to_string(), change: "-0.23%".to_string() },
        Language { rank: 19, prev_rank: 17, name: "Assembly language".to_string(), rating: "1.04%".to_string(), change: "-0.01%".to_string() },
        Language { rank: 20, prev_rank: 23, name: "Kotlin".to_string(), rating: "0.92%".to_string(), change: "+0.10%".to_string() },
    ]
}


fn get_language_detail(name: &str, lang: &Language) -> LanguageDetail {
    let (description, use_cases, frameworks) = match name.to_lowercase().as_str() {
        "python" => (
            "Python 是一种高级、通用的编程语言，以其简洁易读的语法著称。",
            vec!["数据科学", "机器学习", "Web开发", "自动化脚本", "科学计算"],
            vec!["Django", "Flask", "FastAPI", "PyTorch", "TensorFlow", "Pandas"],
        ),
        "c" => (
            "C 是一种通用的过程式编程语言，广泛用于系统编程和嵌入式开发。",
            vec!["操作系统", "嵌入式系统", "驱动程序", "游戏引擎", "数据库"],
            vec!["Linux Kernel", "SQLite", "Git", "Nginx"],
        ),
        "c++" => (
            "C++ 是 C 语言的扩展，支持面向对象编程，广泛用于高性能应用。",
            vec!["游戏开发", "系统软件", "浏览器", "数据库", "图形处理"],
            vec!["Qt", "Boost", "Unreal Engine", "OpenCV"],
        ),
        "java" => (
            "Java 是一种面向对象的编程语言，以其跨平台特性著称。",
            vec!["企业应用", "Android开发", "大数据", "云计算", "微服务"],
            vec!["Spring", "Hibernate", "Maven", "Gradle", "Apache Kafka"],
        ),
        "c#" => (
            "C# 是微软开发的面向对象编程语言，主要用于 .NET 平台开发。",
            vec!["Windows应用", "游戏开发", "Web服务", "企业软件", "云应用"],
            vec![".NET Core", "ASP.NET", "Unity", "Xamarin", "Entity Framework"],
        ),
        "javascript" => (
            "JavaScript 是 Web 开发的核心语言，支持前端和后端开发。",
            vec!["前端开发", "后端开发", "移动应用", "桌面应用", "游戏开发"],
            vec!["React", "Vue.js", "Angular", "Node.js", "Express", "Next.js"],
        ),
        "go" => (
            "Go 是 Google 开发的编程语言，以其简洁和高并发性能著称。",
            vec!["云原生", "微服务", "网络编程", "DevOps工具", "区块链"],
            vec!["Gin", "Echo", "Kubernetes", "Docker", "Prometheus"],
        ),
        "rust" => (
            "Rust 是一种系统编程语言，注重安全性、并发性和性能。",
            vec!["系统编程", "WebAssembly", "嵌入式", "命令行工具", "区块链"],
            vec!["Actix", "Rocket", "Tokio", "Axum", "Diesel"],
        ),
        "php" => (
            "PHP 是一种服务器端脚本语言，广泛用于 Web 开发。",
            vec!["Web开发", "CMS系统", "电商平台", "API开发", "博客系统"],
            vec!["Laravel", "Symfony", "WordPress", "Drupal", "Magento"],
        ),
        "r" => (
            "R 是一种用于统计计算和图形的编程语言。",
            vec!["统计分析", "数据可视化", "机器学习", "生物信息学", "金融分析"],
            vec!["ggplot2", "dplyr", "tidyr", "Shiny", "caret"],
        ),
        "sql" => (
            "SQL 是用于管理关系数据库的标准语言。",
            vec!["数据查询", "数据管理", "报表生成", "数据分析", "ETL"],
            vec!["MySQL", "PostgreSQL", "Oracle", "SQL Server", "SQLite"],
        ),
        "kotlin" => (
            "Kotlin 是 JetBrains 开发的现代编程语言，与 Java 完全兼容。",
            vec!["Android开发", "服务端开发", "跨平台开发", "Web开发"],
            vec!["Ktor", "Spring Boot", "Jetpack Compose", "Exposed"],
        ),
        "visual basic" => (
            "Visual Basic 是微软开发的事件驱动编程语言。",
            vec!["Windows应用", "Office自动化", "数据库应用", "快速原型"],
            vec!["VB.NET", "VBA", "Visual Studio"],
        ),
        "perl" => (
            "Perl 是一种高级、通用的解释型编程语言。",
            vec!["文本处理", "系统管理", "Web开发", "网络编程", "生物信息学"],
            vec!["Mojolicious", "Dancer", "Catalyst", "CPAN"],
        ),
        "delphi/object pascal" | "delphi" => (
            "Delphi/Object Pascal 是一种面向对象的编程语言。",
            vec!["桌面应用", "数据库应用", "跨平台开发", "嵌入式系统"],
            vec!["FireMonkey", "VCL", "RAD Studio"],
        ),
        "fortran" => (
            "Fortran 是最早的高级编程语言之一，主要用于科学计算。",
            vec!["科学计算", "数值分析", "高性能计算", "气象模拟", "物理模拟"],
            vec!["LAPACK", "BLAS", "OpenMP", "MPI"],
        ),
        "matlab" => (
            "MATLAB 是一种用于数值计算的编程语言和环境。",
            vec!["数值计算", "信号处理", "图像处理", "控制系统", "深度学习"],
            vec!["Simulink", "Image Processing Toolbox", "Deep Learning Toolbox"],
        ),
        "ada" => (
            "Ada 是一种结构化、静态类型的编程语言，用于高可靠性系统。",
            vec!["航空航天", "国防系统", "铁路系统", "医疗设备", "嵌入式系统"],
            vec!["GNAT", "SPARK", "Ada Web Server"],
        ),
        "assembly language" | "assembly" => (
            "汇编语言是一种低级编程语言，与机器码直接对应。",
            vec!["操作系统", "驱动程序", "嵌入式系统", "逆向工程", "性能优化"],
            vec!["NASM", "MASM", "GAS"],
        ),
        "scratch" => (
            "Scratch 是一种可视化编程语言，主要用于编程教育。",
            vec!["编程教育", "游戏开发", "动画制作", "互动故事"],
            vec!["Scratch 3.0", "ScratchJr"],
        ),
        _ => (
            "这是一种流行的编程语言。",
            vec!["通用编程"],
            vec!["暂无"],
        ),
    };

    LanguageDetail {
        name: lang.name.clone(),
        rank: lang.rank,
        rating: lang.rating.clone(),
        description: description.to_string(),
        use_cases: use_cases.iter().map(|s| s.to_string()).collect(),
        frameworks: frameworks.iter().map(|s| s.to_string()).collect(),
    }
}

#[utoipa::path(
    get,
    path = "/api/languages",
    params(DateQuery),
    responses(
        (status = 200, description = "List of languages", body = Vec<Language>)
    )
)]
async fn get_languages(Query(params): Query<DateQuery>) -> Result<Json<Vec<Language>>, StatusCode> {
    match fetch_tiobe_data(params.year, params.month).await {
        Ok(languages) => Ok(Json(languages)),
        Err(_) => Ok(Json(get_fallback_data())),
    }
}

#[utoipa::path(
    get,
    path = "/api/language/{name}",
    params(
        ("name" = String, Path, description = "Language name"),
        DateQuery
    ),
    responses(
        (status = 200, description = "Language details", body = LanguageDetail),
        (status = 404, description = "Language not found")
    )
)]
async fn get_language_info(
    Path(name): Path<String>,
    Query(params): Query<DateQuery>,
) -> Result<Json<LanguageDetail>, StatusCode> {
    let languages = fetch_tiobe_data(params.year, params.month)
        .await
        .unwrap_or_else(|_| get_fallback_data());
    
    if let Some(lang) = languages.iter().find(|l| l.name.to_lowercase() == name.to_lowercase()) {
        Ok(Json(get_language_detail(&name, lang)))
    } else {
        let default_lang = Language {
            rank: 0,
            prev_rank: 0,
            name: name.clone(),
            rating: "N/A".to_string(),
            change: "N/A".to_string(),
        };
        Ok(Json(get_language_detail(&name, &default_lang)))
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/languages", get(get_languages))
        .route("/api/language/:name", get(get_language_info))
        .nest_service("/", ServeDir::new("static"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
