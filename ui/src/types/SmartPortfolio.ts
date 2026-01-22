export interface SmartPortfolioMessage {
  author: string
  content: string
}

export interface NewMessage {
  smart_portfolio: string
  author: string
  content: string
}

export interface SendSmartPortfolioMessage {
  Send: {
    target: string
    message: string
  }
}

// SmartPortfolios consists of a map of counterparty to an array of messages
export interface SmartPortfolios {
  [counterparty: string]: SmartPortfolioMessage[]
}
