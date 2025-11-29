import { redirect } from "@sveltejs/kit";
import type { LayoutLoad } from "../$types";

export const load: LayoutLoad = async ({ route, url }) => {
  if (route.id === "/(space)") {
    redirect(302, "/new");
  }
};
