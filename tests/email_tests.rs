mod tests {
    use super::super::email_extractor::rank_local_part;

    #[test]
    fn ranks_contact_info_sales() {
        assert!(rank_local_part("contact") < rank_local_part("info"));
        assert!(rank_local_part("info") < rank_local_part("sales"));
        assert!(rank_local_part("sales") < rank_local_part("zach"));
    }
}
