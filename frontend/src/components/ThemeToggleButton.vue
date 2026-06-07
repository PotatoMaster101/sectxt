<template>
  <Button @click="toggle" :icon="isDark ? 'pi pi-sun' : 'pi pi-moon'" aria-label="Toggle Dark Mode" rounded text />
</template>

<script setup lang="ts">
import { ref } from 'vue';

const isDark = ref(true);

const toggle = () => {
  isDark.value = !isDark.value;
  if (isDark.value) {
    document.documentElement.classList.add('dark-mode');
  } else {
    document.documentElement.classList.remove('dark-mode');
  }
};
</script>
