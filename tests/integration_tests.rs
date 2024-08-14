// // use std::path::Path;
// use tempfile::TempDir;
// use tokio::fs;
// use widots::config::AppConfig;
// use widots::error::AppError;

// #[tokio::test]
// async fn test_full_workflow() -> Result<(), AppError> {
//     // テスト用の一時ディレクトリを作成
//     let temp_dir = TempDir::new()?;
//     let source_dir = temp_dir.path().join("source");
//     let target_dir = temp_dir.path().join("target");
//     fs::create_dir_all(&source_dir).await?;
//     fs::create_dir_all(&target_dir).await?;

//     // テスト用のドットファイルを作成
//     fs::write(source_dir.join(".testrc"), "test content").await?;
//     fs::write(source_dir.join(".vimrc"), "vim config").await?;
//     fs::write(source_dir.join(".zshrc"), "zsh config").await?;

//     // AppConfig を初期化
//     let config = AppConfig::new().await?;

//     // リンク作成のテスト
//     let link_results = config
//         .get_link_service()
//         .link_dotfiles(&source_dir, &target_dir, false)
//         .await?;

//     // リンクが正しく作成されたか確認
//     assert!(target_dir.join(".testrc").is_symlink());
//     assert!(target_dir.join(".vimrc").is_symlink());
//     assert!(target_dir.join(".zshrc").is_symlink());

//     // リンク結果の検証
//     assert_eq!(link_results.len(), 3);
//     for result in link_results {
//         match result {
//             widots::models::link::FileProcessResult::Linked(src, dst) => {
//                 println!("Linked: {} -> {}", src.display(), dst.display());
//                 assert!(src.exists());
//                 assert!(dst.is_symlink());
//             }
//             _ => panic!("Unexpected result: {:?}", result),
//         }
//     }

//     // 設定のロードテスト（仮の設定ファイルを作成）
//     let config_content = r#"
//         [[link]]
//         location = "source"

//         [[provision]]
//         mode = "test"
//         script = "echo 'This is a test provision'"
//     "#;
//     let config_path = temp_dir.path().join("config.toml");
//     fs::write(&config_path, config_content).await?;

//     config
//         .get_load_service()
//         .load(&config_path, &target_dir, false)
//         .await?;

//     // Brew サービスのテスト（実際のインストールは行わず、モック動作のみ）
//     let brew_result = config.get_brew_service().export().await;
//     assert!(
//         brew_result.is_ok(),
//         "Brew export failed: {:?}",
//         brew_result.err()
//     );

//     // Fish サービスのテスト（実際のインストールは行わず、モック動作のみ）
//     let fish_result = config.get_fish_service().install().await;
//     assert!(
//         fish_result.is_ok(),
//         "Fish install failed: {:?}",
//         fish_result.err()
//     );

//     // VSCode サービスのテスト（実際の操作は行わず、モック動作のみ）
//     let vscode_result = config.get_vscode_service().export_extensions().await;
//     assert!(
//         vscode_result.is_ok(),
//         "VSCode export failed: {:?}",
//         vscode_result.err()
//     );

//     // マテリアライズのテスト
//     let materialize_results = config
//         .get_link_service()
//         .materialize_dotfiles(&target_dir)
//         .await?;

//     // マテリアライズ結果の検証
//     assert_eq!(materialize_results.len(), 3);
//     for result in materialize_results {
//         match result {
//             widots::models::link::FileProcessResult::Materialized(path, original) => {
//                 println!(
//                     "Materialized: {} (was linked to {})",
//                     path.display(),
//                     original.display()
//                 );
//                 assert!(path.is_file());
//                 assert!(!path.is_symlink());
//             }
//             _ => panic!("Unexpected result: {:?}", result),
//         }
//     }

//     // ファイル内容の検証
//     let testrc_content = fs::read_to_string(target_dir.join(".testrc")).await?;
//     assert_eq!(testrc_content, "test content");

//     let vimrc_content = fs::read_to_string(target_dir.join(".vimrc")).await?;
//     assert_eq!(vimrc_content, "vim config");

//     let zshrc_content = fs::read_to_string(target_dir.join(".zshrc")).await?;
//     assert_eq!(zshrc_content, "zsh config");

//     // デプロイサービスのテスト（実際のデプロイは行わず、モック動作のみ）
//     let deploy_result = config.get_deploy_service().execute().await;
//     assert!(
//         deploy_result.is_ok(),
//         "Deploy failed: {:?}",
//         deploy_result.err()
//     );

//     println!("All integration tests passed successfully!");
//     Ok(())
// }
