import { create } from "zustand";
import type { ProductContract } from "../types/productContract";

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

interface ProductStoreState {
  // Data
  productContract: ProductContract | null;
  loaded: boolean;

  // Actions
  setProductContract: (contract: ProductContract) => void;
  reset: () => void;
}

export const useProductStore = create<ProductStoreState>((set) => ({
  productContract: null,
  loaded: false,

  setProductContract: (contract: ProductContract) =>
    set({ productContract: contract, loaded: true }),

  reset: () =>
    set({ productContract: null, loaded: false }),
}));
