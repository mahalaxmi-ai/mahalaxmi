import {
  SITE_URL,
  COMPANY_NAME,
  DEFAULT_OG_IMAGE,
  CONTACT_EMAIL,
  COMPANY_PHONE,
  COMPANY_ADDRESS,
} from './seoConstants';

export function organizationSchema() {
  return {
    '@context': 'https://schema.org',
    '@type': 'Organization',
    name: COMPANY_NAME,
    url: SITE_URL,
    logo: DEFAULT_OG_IMAGE,
    contactPoint: {
      '@type': 'ContactPoint',
      email: CONTACT_EMAIL,
      telephone: COMPANY_PHONE,
      contactType: 'customer service',
      availableLanguage: ['English', 'Spanish', 'French', 'German', 'Portuguese', 'Japanese', 'Chinese', 'Korean', 'Hindi', 'Arabic'],
    },
    address: {
      '@type': 'PostalAddress',
      ...COMPANY_ADDRESS,
    },
  };
}

export function localBusinessSchema() {
  return {
    '@context': 'https://schema.org',
    '@type': 'LocalBusiness',
    '@id': `${SITE_URL}/#localbusiness`,
    name: COMPANY_NAME,
    url: SITE_URL,
    image: DEFAULT_OG_IMAGE,
    email: CONTACT_EMAIL,
    telephone: COMPANY_PHONE,
    address: {
      '@type': 'PostalAddress',
      ...COMPANY_ADDRESS,
    },
    openingHoursSpecification: {
      '@type': 'OpeningHoursSpecification',
      dayOfWeek: ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday'],
      opens: '09:00',
      closes: '18:00',
    },
    priceRange: '$$',
  };
}

export function websiteSchema() {
  return {
    '@context': 'https://schema.org',
    '@type': 'WebSite',
    name: COMPANY_NAME,
    url: SITE_URL,
    potentialAction: {
      '@type': 'SearchAction',
      target: {
        '@type': 'EntryPoint',
        urlTemplate: `${SITE_URL}/docs/{productSlug}/search?q={search_term_string}`,
      },
      'query-input': 'required name=search_term_string',
    },
  };
}

export function softwareApplicationSchema(product) {
  return {
    '@context': 'https://schema.org',
    '@type': 'SoftwareApplication',
    name: product.name,
    description: product.short_description,
    url: `${SITE_URL}/products/${product.slug}`,
    applicationCategory: 'BusinessApplication',
    operatingSystem: product.specifications?.Platform || 'Windows, macOS, Linux',
    ...(product.image && { image: product.image }),
    ...(product.pricing_options?.length > 0 && {
      offers: product.pricing_options.map((option) => ({
        '@type': 'Offer',
        price: option.price || 0,
        priceCurrency: 'USD',
        name: option.name,
      })),
    }),
  };
}

export function productSchema(product) {
  return {
    '@context': 'https://schema.org',
    '@type': 'Product',
    name: product.name,
    description: product.short_description,
    url: `${SITE_URL}/products/${product.slug}`,
    brand: {
      '@type': 'Organization',
      name: COMPANY_NAME,
    },
    ...(product.image && { image: product.image }),
    ...(product.pricing_options?.length > 0 && {
      offers: product.pricing_options.map((option) => ({
        '@type': 'Offer',
        price: option.price || 0,
        priceCurrency: 'USD',
        name: option.name,
        availability: 'https://schema.org/InStock',
      })),
    }),
  };
}

export function faqPageSchema(faqs) {
  return {
    '@context': 'https://schema.org',
    '@type': 'FAQPage',
    mainEntity: faqs.map((faq) => ({
      '@type': 'Question',
      name: faq.question,
      acceptedAnswer: {
        '@type': 'Answer',
        text: faq.answer,
      },
    })),
  };
}

export function breadcrumbSchema(items) {
  return {
    '@context': 'https://schema.org',
    '@type': 'BreadcrumbList',
    itemListElement: items.map((item, index) => ({
      '@type': 'ListItem',
      position: index + 1,
      name: item.name,
      ...(item.url && { item: `${SITE_URL}${item.url}` }),
    })),
  };
}

export function articleSchema(post) {
  return {
    '@context': 'https://schema.org',
    '@type': 'Article',
    headline: post.title,
    description: post.description,
    url: `${SITE_URL}/blog/${post.slug}`,
    datePublished: post.date,
    author: {
      '@type': 'Organization',
      name: COMPANY_NAME,
      url: SITE_URL,
    },
    publisher: {
      '@type': 'Organization',
      name: COMPANY_NAME,
      url: SITE_URL,
      logo: {
        '@type': 'ImageObject',
        url: DEFAULT_OG_IMAGE,
      },
    },
    ...(post.keywords && { keywords: post.keywords }),
  };
}

export function blogSchema() {
  return {
    '@context': 'https://schema.org',
    '@type': 'Blog',
    name: `${COMPANY_NAME} Blog`,
    description: 'Technology insights, guides, and best practices from ThriveTech Services.',
    url: `${SITE_URL}/blog`,
    publisher: {
      '@type': 'Organization',
      name: COMPANY_NAME,
      url: SITE_URL,
    },
  };
}

export function techArticleSchema(section, productSlug, productName) {
  return {
    '@context': 'https://schema.org',
    '@type': 'TechArticle',
    headline: section.title,
    description: section.metadata?.description || `${section.title} - ${productName} documentation`,
    url: `${SITE_URL}/docs/${productSlug}/manual/${section.slug}`,
    author: {
      '@type': 'Organization',
      name: COMPANY_NAME,
      url: SITE_URL,
    },
    publisher: {
      '@type': 'Organization',
      name: COMPANY_NAME,
      url: SITE_URL,
      logo: {
        '@type': 'ImageObject',
        url: DEFAULT_OG_IMAGE,
      },
    },
    ...(section.metadata?.lastUpdated && { dateModified: section.metadata.lastUpdated }),
    ...(section.metadata?.tags && { keywords: section.metadata.tags.join(', ') }),
  };
}

export function serviceSchema(service) {
  return {
    '@context': 'https://schema.org',
    '@type': 'Service',
    name: service.name,
    description: service.description,
    url: `${SITE_URL}${service.url}`,
    provider: {
      '@type': 'Organization',
      name: COMPANY_NAME,
      url: SITE_URL,
    },
    areaServed: {
      '@type': 'Country',
      name: 'United States',
    },
    ...(service.serviceType && { serviceType: service.serviceType }),
  };
}
