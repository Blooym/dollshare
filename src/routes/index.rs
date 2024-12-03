pub async fn index_handler() -> &'static str {
    concat!(
        env!("CARGO_PKG_NAME"),
        " - ",
        env!("CARGO_PKG_DESCRIPTION"),
        "\n\n",
        env!("CARGO_PKG_REPOSITORY")
    )
}
