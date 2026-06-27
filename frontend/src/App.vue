<template>
  <div class="h-full w-full">
    <header class="flex items-center justify-center mt-4 w-full">
      <ThemeToggleButton />
    </header>

    <main class="m-4">
      <ConfirmDialog />
      <Toast />
      <RouterView />
    </main>
  </div>
</template>
