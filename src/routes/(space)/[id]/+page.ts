import { error, redirect } from "@sveltejs/kit";
import type { PageLoad } from "./$types";
import { getAppState } from "$lib/stores/state.svelte";

export const load: PageLoad = async ({ params }) => {
  const appState = getAppState();

  const space = appState.spaces.get(params.id);

  if (!space) {
    console.log("Space not found, redirecting to new");
    redirect(302, "/new");
  }

  appState.selectedSpaceId = params.id;

  return {
    id: params.id,
  };
};
