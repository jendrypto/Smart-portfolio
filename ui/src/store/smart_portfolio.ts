import { create } from 'zustand'
import { NewMessage, SmartPortfolios } from '../types/SmartPortfolio'
import { persist, createJSONStorage } from 'zustand/middleware'

export interface SmartPortfolioStore {
  smart_portfolios: SmartPortfolios
  addMessage: (msg: NewMessage) => void
  get: () => SmartPortfolioStore
  set: (partial: SmartPortfolioStore | Partial<SmartPortfolioStore>) => void
}

const useSmartPortfolioStore = create<SmartPortfolioStore>()(
  persist(
    (set, get) => ({
      smart_portfolios: { "New SmartPortfolio": [] },
      addMessage: (msg: NewMessage) => {
        const { smart_portfolios } = get()
        const { smart_portfolio, author, content } = msg
        if (!smart_portfolios[smart_portfolio]) {
          smart_portfolios[smart_portfolio] = []
        }
        smart_portfolios[smart_portfolio].push({ author, content })
        set({ smart_portfolios })
      },

      get,
      set,
    }),
    {
      name: 'smart_portfolio', // unique name
      storage: createJSONStorage(() => sessionStorage), // (optional) by default, 'localStorage' is used
    }
  )
)

export default useSmartPortfolioStore
