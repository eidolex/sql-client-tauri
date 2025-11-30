import { error, redirect } from "@sveltejs/kit";
import type { PageLoad } from "./$types";
import { getAppState } from "$lib/stores/state.svelte";

export const load: PageLoad = async ({ params }) => {
  const appState = getAppState();

  const space = appState.spaces.get(params.id);

  if (!space) {
    redirect(302, "/new");
  }

  return {
    id: params.id,
  };
};
