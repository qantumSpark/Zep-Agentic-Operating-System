import { create } from "zustand";

export interface ProjectStoreState {
  projectDir: string;
  projectName: string;
  isLoading: boolean;

  // Actions
  setProject: (dir: string, name: string) => void;
  setLoading: (loading: boolean) => void;
  reset: () => void;
}

export const useProjectStore = create<ProjectStoreState>((set) => ({
  projectDir: "",
  projectName: "",
  isLoading: false,

  setProject: (dir: string, name: string) =>
    set({
      projectDir: dir,
      projectName: name,
    }),

  setLoading: (loading: boolean) =>
    set({
      isLoading: loading,
    }),

  reset: () =>
    set({
      projectDir: "",
      projectName: "",
      isLoading: false,
    }),
}));
