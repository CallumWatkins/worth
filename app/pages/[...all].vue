<script lang="ts" setup>
throw createError({
  status: 404,
  statusText: "Page not found",
  message: "The page you are looking for does not exist.",
  fatal: true
});
</script>
