import axios from 'axios';

const apiClient = axios.create({ baseURL: '/api', timeout: 10000 });

export const productsAPI = {
  getAll: (params = {}) => apiClient.get('/products', { params }),
  getBySlug: (slug) => apiClient.get(`/products/${slug}`),
};

export const categoriesAPI = {
  getAll: () => apiClient.get('/categories'),
};

export const releasesAPI = {
  getLatest: () => apiClient.get('/releases/latest'),
};

export const checkoutAPI = {
  getPricing: () => apiClient.get('/checkout'),
  createSession: (data) => apiClient.post('/checkout', data),
  getSession: (id) => apiClient.get(`/checkout/session/${id}`),
};

export const serversAPI = {
  getAll: () => apiClient.get('/mahalaxmi/servers'),
  getById: (id) => apiClient.get(`/mahalaxmi/servers/${id}`),
  configure: (id, data) => apiClient.patch(`/mahalaxmi/servers/${id}/configure`, data),
  getVscodeConfig: (id) => apiClient.get(`/mahalaxmi/servers/${id}/vscode-config`),
  deleteProject: (id) => apiClient.delete(`/mahalaxmi/projects/${id}`),
};

export const billingAPI = {
  getPortalUrl: () => apiClient.get('/mahalaxmi/billing/portal-url'),
};

export const authAPI = {
  me: () => apiClient.get('/auth/me'),
  register: (data) => apiClient.post('/auth/register', data),
  login: (data) => apiClient.post('/auth/login', data),
  logout: () => apiClient.post('/auth/logout'),
  forgotPassword: (data) => apiClient.post('/auth/forgot-password', data),
  resetPassword: (data) => apiClient.post('/auth/reset-password', data),
  resendVerification: (data) => apiClient.post('/auth/resend-verification', data),
};

export default apiClient;
