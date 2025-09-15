<template>
  <div class="app-container">
    <router-view />
  </div>
</template>

<script>
export default {
  name: "App",
};
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen,
    Ubuntu, Cantarell, "Open Sans", "Helvetica Neue", sans-serif;
  background-color: #f5f7fa;
  color: #323233;
  font-size: 14px;
}

.app-container {
  min-height: 100vh;
}
</style>
