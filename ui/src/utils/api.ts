// Use Vite's BASE_URL for API calls - ensures correct path when deployed to Hyperware
export const API_BASE = import.meta.env.BASE_URL.replace(/\/$/, ''); // Remove trailing slash if present
