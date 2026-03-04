<script lang="ts" setup>
throw createError({
  statusCode: 404,
  statusMessage: "Page not found",
  message: "The page you are looking for does not exist.",
  fatal: true
});
</script>
