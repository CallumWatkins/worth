export default defineAppConfig({
  ui: {
    colors: {
      primary: "green",
      neutral: "zinc"
    },
    button: {
      slots: {
        base: "cursor-pointer"
      }
    },
    selectMenu: {
      variants: {
        variant: {
          soft: "focus:bg-elevated/50 focus:hover:bg-elevated"
        }
      }
    },
    pageBody: {
      base: "pb-12"
    },
    breadcrumb: {
      slots: {
        item: "[&:first-child]:min-w-auto [&:not(:first-child)]:min-w-[calc-size(max-content,min(size,calc(var(--spacing)*30)))]"
      }
    }
  }
});
