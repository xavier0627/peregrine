use anyhow::Result; 
use std::path::Path; 

pub fn run_init(path: &Path) -> Result<()> { 
    if path.exists() { 
        anyhow::bail!("config file already exists: {}", path.display()); 
    }

    let text = default_config_text(); 

    std::fs::write(path, text)?; 

    tracing::info!(path = %path.display(), "config file created"); 

    Ok(()) 
}

fn default_config_text() -> &'static str { 
    concat!(
        "listen: 0.0.0.0:8080 # HTTP 服务监听地址\n",
        "healthPath: /__health__ # 健康检查路径\n",
        "staticDir: ./dist # 静态文件目录\n",
        "securityHeaders: true # 是否自动添加安全响应头\n",
        "proxy: # 代理路由表\n",
        "  /api: http://127.0.0.1:4000 # /api 和 /api/* 转发到 4000\n",
    )
}

#[cfg(test)]
mod tests {
    use super::default_config_text;

    #[test]
    fn default_config_text_uses_root_level_top_level_keys() {
        let expected = concat!(
            "listen: 0.0.0.0:8080 # HTTP 服务监听地址\n",
            "healthPath: /__health__ # 健康检查路径\n",
            "staticDir: ./dist # 静态文件目录\n",
            "\n",
            "proxy: # 代理路由表\n",
            "  /api: http://127.0.0.1:4000 # /api 和 /api/* 转发到 4000\n",
        );

        assert_eq!(default_config_text(), expected);
    }
}
