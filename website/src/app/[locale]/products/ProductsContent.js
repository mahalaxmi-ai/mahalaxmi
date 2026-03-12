'use client';

import React, { useEffect } from 'react';
import Image from 'next/image';
import {
  Container,
  Typography,
  Grid,
  Card,
  CardContent,
  CardActionArea,
  Button,
  Box,
  Chip,
  alpha,
  useTheme,
  Fade,
  Grow,
  Breadcrumbs,
  Link as MuiLink,
} from '@mui/material';
import {
  Psychology,
  Code,
  Business,
  ArrowForward,
  ArrowBack,
  Rocket,
  AutoAwesome,
  CheckCircle,
} from '@mui/icons-material';
import Link from 'next/link';
import { useSearchParams } from 'next/navigation';
import { useQuery } from '@tanstack/react-query';
import JsonLd from '@/components/SEO/JsonLd';
import { breadcrumbSchema } from '@/utils/seoSchemas';
import { productsAPI, categoriesAPI } from '@/lib/api';
import LoadingSpinner from '@/components/UI/LoadingSpinner';

const ProductsContent = () => {
  const theme = useTheme();
  const searchParams = useSearchParams();
  const selectedCategorySlug = searchParams.get('category');

  const handleCategorySelect = (slug) => {
    const params = new URLSearchParams();
    params.set('category', slug);
    window.history.pushState(null, '', `?${params.toString()}`);
    window.dispatchEvent(new Event('popstate'));
  };

  const handleBackToCategories = () => {
    window.history.pushState(null, '', '/products');
    window.dispatchEvent(new Event('popstate'));
  };

  const { data: categoriesData, isLoading: categoriesLoading } = useQuery({
    queryKey: ['categories'],
    queryFn: categoriesAPI.getAll,
  });

  const categories = categoriesData?.data?.data || [];
  const selectedCategory = categories.find(c => c.slug === selectedCategorySlug);

  // Redirect if category has an external link (e.g., Enterprise Solutions -> /enterprise)
  useEffect(() => {
    if (selectedCategory?.external_link) {
      window.location.href = selectedCategory.external_link;
    }
  }, [selectedCategory]);

  const { data: productsData, isLoading: productsLoading } = useQuery({
    queryKey: ['products', { category: selectedCategory?.id }],
    queryFn: () => productsAPI.getAll({ category: selectedCategory?.id }),
    enabled: !!selectedCategory,
  });

  const products = productsData?.data?.data?.products || [];

  const getCategoryIcon = (iconName) => {
    const iconMap = {
      psychology: <Psychology sx={{ fontSize: 48 }} />,
      code: <Code sx={{ fontSize: 48 }} />,
      business: <Business sx={{ fontSize: 48 }} />,
    };
    return iconMap[iconName] || <Rocket sx={{ fontSize: 48 }} />;
  };

  // Categories View
  const CategoriesView = () => (
    <>
      {/* Hero Section */}
      <Box
        sx={{
          background: `linear-gradient(135deg, ${alpha(theme.palette.primary.dark, 0.95)} 0%, ${alpha('#1a1a2e', 0.98)} 100%)`,
          color: 'white',
          py: { xs: 8, md: 12 },
          position: 'relative',
          overflow: 'hidden',
        }}
      >
        <Container maxWidth="lg" sx={{ position: 'relative', zIndex: 1 }}>
          <Fade in timeout={800}>
            <Box sx={{ textAlign: 'center' }}>
              <Box sx={{ display: 'inline-flex', alignItems: 'center', mb: 2 }}>
                <AutoAwesome sx={{ mr: 1, color: '#ffd700' }} />
                <Typography variant="overline" sx={{ letterSpacing: 3, color: '#ffd700' }}>
                  Product Categories
                </Typography>
              </Box>
              <Typography
                variant="h1"
                component="h1"
                sx={{
                  fontWeight: 800,
                  fontSize: { xs: '2.5rem', md: '4rem' },
                  mb: 3,
                }}
              >
                Our Solutions
              </Typography>
              <Typography
                variant="h5"
                sx={{
                  maxWidth: 700,
                  mx: 'auto',
                  opacity: 0.9,
                  fontWeight: 300,
                  lineHeight: 1.6,
                }}
              >
                Explore our product categories to find the perfect solution for your needs
              </Typography>
            </Box>
          </Fade>
        </Container>
      </Box>

      {/* Categories Grid */}
      <Container maxWidth="lg" sx={{ py: { xs: 6, md: 10 } }}>
        {categoriesLoading ? (
          <LoadingSpinner size={60} />
        ) : (
          <Grid container spacing={4}>
            {categories.map((category, index) => (
              <Grid item xs={12} md={4} key={category.id}>
                <Grow in timeout={600 + index * 150}>
                  <Card
                    sx={{
                      height: '100%',
                      borderRadius: 4,
                      border: '1px solid',
                      borderColor: 'divider',
                      position: 'relative',
                      overflow: 'hidden',
                      transition: 'all 0.4s cubic-bezier(0.4, 0, 0.2, 1)',
                      opacity: category.coming_soon ? 0.7 : 1,
                      '&:hover': {
                        transform: category.coming_soon ? 'none' : 'translateY(-12px)',
                        boxShadow: category.coming_soon ? 'none' : `0 25px 50px ${alpha(category.color || theme.palette.primary.main, 0.25)}`,
                        borderColor: category.coming_soon ? 'divider' : category.color || 'primary.main',
                      },
                    }}
                  >
                    <CardActionArea
                      onClick={() => {
                        if (category.coming_soon) return;
                        if (category.external_link) {
                          window.location.href = category.external_link;
                        } else {
                          handleCategorySelect(category.slug);
                        }
                      }}
                      disabled={category.coming_soon}
                      sx={{ height: '100%', p: 0 }}
                    >
                      {/* Category Header */}
                      <Box
                        sx={{
                          height: 180,
                          background: category.image
                            ? `linear-gradient(135deg, ${alpha(category.color || theme.palette.primary.main, 0.9)} 0%, ${alpha(category.color || theme.palette.primary.dark, 0.95)} 100%)`
                            : `linear-gradient(135deg, ${alpha(category.color || theme.palette.primary.main, 0.1)} 0%, ${alpha(category.color || theme.palette.primary.main, 0.2)} 100%)`,
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                          position: 'relative',
                        }}
                      >
                        {category.image ? (
                          <Image
                            src={category.image}
                            alt={category.name}
                            fill
                            sizes="(max-width: 600px) 50vw, 200px"
                            style={{ objectFit: 'contain', padding: '15%' }}
                            unoptimized
                          />
                        ) : (
                          <Box sx={{ color: category.color || 'primary.main' }}>
                            {getCategoryIcon(category.icon)}
                          </Box>
                        )}
                        {category.coming_soon && (
                          <Chip
                            label="Coming Soon"
                            size="small"
                            sx={{
                              position: 'absolute',
                              top: 12,
                              right: 12,
                              bgcolor: 'grey.800',
                              color: 'white',
                            }}
                          />
                        )}
                        {category.product_count > 0 && (
                          <Chip
                            label={`${category.product_count} Product${category.product_count > 1 ? 's' : ''}`}
                            size="small"
                            sx={{
                              position: 'absolute',
                              top: 12,
                              right: 12,
                              bgcolor: category.color || 'primary.main',
                              color: 'white',
                            }}
                          />
                        )}
                      </Box>

                      <CardContent sx={{ p: 4 }}>
                        <Typography
                          variant="h4"
                          component="h2"
                          sx={{ fontWeight: 700, mb: 2 }}
                        >
                          {category.name}
                        </Typography>
                        <Typography
                          variant="body1"
                          color="text.secondary"
                          sx={{ mb: 3, lineHeight: 1.7 }}
                        >
                          {category.description}
                        </Typography>
                        {!category.coming_soon && (
                          <Box sx={{ display: 'flex', alignItems: 'center', color: category.color || 'primary.main' }}>
                            <Typography variant="body1" sx={{ fontWeight: 600, mr: 1 }}>
                              {category.cta_text || 'Explore Products'}
                            </Typography>
                            <ArrowForward fontSize="small" />
                          </Box>
                        )}
                      </CardContent>
                    </CardActionArea>
                  </Card>
                </Grow>
              </Grid>
            ))}
          </Grid>
        )}
      </Container>
    </>
  );

  // Products in Category View
  const ProductsInCategoryView = () => (
    <>
      {/* Hero Section */}
      <Box
        sx={{
          background: `linear-gradient(135deg, ${alpha(selectedCategory?.color || theme.palette.primary.dark, 0.95)} 0%, ${alpha('#1a1a2e', 0.98)} 100%)`,
          color: 'white',
          py: { xs: 6, md: 10 },
          position: 'relative',
        }}
      >
        <Container maxWidth="lg" sx={{ position: 'relative', zIndex: 1 }}>
          {/* Breadcrumbs */}
          <Breadcrumbs
            sx={{
              mb: 4,
              '& .MuiBreadcrumbs-separator': { color: 'rgba(255,255,255,0.5)' },
            }}
          >
            <MuiLink
              component={Link}
              href="/"
              underline="hover"
              sx={{ color: 'rgba(255,255,255,0.7)', '&:hover': { color: 'white' } }}
            >
              Home
            </MuiLink>
            <MuiLink
              component="button"
              underline="hover"
              onClick={handleBackToCategories}
              sx={{
                color: 'rgba(255,255,255,0.7)',
                '&:hover': { color: 'white' },
                background: 'none',
                border: 'none',
                cursor: 'pointer',
                font: 'inherit',
              }}
            >
              Products
            </MuiLink>
            <Typography sx={{ color: 'white' }}>{selectedCategory?.name}</Typography>
          </Breadcrumbs>

          <Button
            startIcon={<ArrowBack />}
            onClick={handleBackToCategories}
            sx={{
              color: 'white',
              mb: 3,
              '&:hover': { bgcolor: 'rgba(255,255,255,0.1)' },
            }}
          >
            All Categories
          </Button>

          <Fade in timeout={800}>
            <Box>
              <Typography
                variant="h1"
                component="h1"
                sx={{
                  fontWeight: 800,
                  fontSize: { xs: '2.5rem', md: '3.5rem' },
                  mb: 2,
                }}
              >
                {selectedCategory?.name}
              </Typography>
              <Typography
                variant="h5"
                sx={{
                  maxWidth: 700,
                  opacity: 0.9,
                  fontWeight: 300,
                  lineHeight: 1.6,
                }}
              >
                {selectedCategory?.long_description || selectedCategory?.description}
              </Typography>
            </Box>
          </Fade>
        </Container>
      </Box>

      {/* Products Grid */}
      <Container maxWidth="lg" sx={{ py: { xs: 6, md: 10 } }}>
        {productsLoading ? (
          <LoadingSpinner size={60} />
        ) : products.length === 0 ? (
          <Box sx={{ textAlign: 'center', py: 8 }}>
            <Typography variant="h5" color="text.secondary">
              No products available in this category yet.
            </Typography>
          </Box>
        ) : (
          <Grid container spacing={4}>
            {products.map((product, index) => (
              <Grid item xs={12} key={product.id}>
                <Grow in timeout={600 + index * 150}>
                  <Card
                    sx={{
                      borderRadius: 4,
                      border: '1px solid',
                      borderColor: 'divider',
                      overflow: 'hidden',
                      transition: 'all 0.4s cubic-bezier(0.4, 0, 0.2, 1)',
                      '&:hover': {
                        boxShadow: `0 25px 50px ${alpha(theme.palette.primary.main, 0.15)}`,
                        borderColor: 'primary.main',
                      },
                    }}
                  >
                    <Grid container>
                      {/* Product Image */}
                      <Grid item xs={12} md={4}>
                        <Box
                          sx={{
                            height: { xs: 200, md: '100%' },
                            minHeight: { md: 300 },
                            background: `linear-gradient(135deg, ${alpha(theme.palette.primary.main, 0.1)} 0%, ${alpha(theme.palette.secondary.main, 0.1)} 100%)`,
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            position: 'relative',
                            p: 4,
                          }}
                        >
                          {product.image ? (
                            <Image
                              src={product.image}
                              alt={product.name}
                              fill
                              sizes="(max-width: 600px) 80vw, 300px"
                              style={{ objectFit: 'contain' }}
                              unoptimized
                            />
                          ) : (
                            <Rocket sx={{ fontSize: 100, color: 'primary.main', opacity: 0.5 }} />
                          )}
                        </Box>
                      </Grid>

                      {/* Product Info */}
                      <Grid item xs={12} md={8}>
                        <CardContent sx={{ p: { xs: 3, md: 5 } }}>
                          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mb: 2, flexWrap: 'wrap' }}>
                            {product.is_featured && (
                              <Chip
                                icon={<AutoAwesome sx={{ fontSize: 16 }} />}
                                label="Featured"
                                size="small"
                                sx={{
                                  background: 'linear-gradient(90deg, #ffd700 0%, #ffaa00 100%)',
                                  color: '#000',
                                  fontWeight: 600,
                                }}
                              />
                            )}
                            {product.pricing_options_count > 0 && (
                              <Chip
                                label={`${product.pricing_options_count} Pricing Options`}
                                size="small"
                                variant="outlined"
                                color="primary"
                              />
                            )}
                          </Box>

                          <Typography variant="h3" sx={{ fontWeight: 700, mb: 1 }}>
                            {product.name}
                          </Typography>

                          {product.tagline && (
                            <Typography
                              variant="h6"
                              color="primary"
                              sx={{ mb: 2, fontWeight: 500 }}
                            >
                              {product.tagline}
                            </Typography>
                          )}

                          <Typography
                            variant="body1"
                            color="text.secondary"
                            sx={{ mb: 3, lineHeight: 1.8 }}
                          >
                            {product.short_description}
                          </Typography>

                          {/* Features Preview */}
                          {product.features && product.features.length > 0 && (
                            <Box sx={{ mb: 3 }}>
                              <Grid container spacing={1}>
                                {product.features.map((feature, idx) => (
                                  <Grid item xs={12} sm={6} key={idx}>
                                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                                      <CheckCircle sx={{ fontSize: 18, color: 'success.main' }} />
                                      <Typography variant="body2">{typeof feature === 'object' ? feature.name : feature}</Typography>
                                    </Box>
                                  </Grid>
                                ))}
                              </Grid>
                            </Box>
                          )}

                          {/* Pricing Preview & CTA */}
                          <Box sx={{ display: 'flex', alignItems: 'center', gap: 3, flexWrap: 'wrap' }}>
                            {product.starting_price && product.pricing_options_count > 0 && (
                              <Box>
                                <Typography variant="caption" color="text.secondary">
                                  Starting at
                                </Typography>
                                <Typography variant="h4" sx={{ fontWeight: 700, color: 'primary.main' }}>
                                  ${product.starting_price}
                                  <Typography component="span" variant="body2" color="text.secondary">
                                    /mo
                                  </Typography>
                                </Typography>
                              </Box>
                            )}
                            <Button
                              component={Link}
                              href={`/products/${product.slug}`}
                              variant="contained"
                              size="large"
                              endIcon={<ArrowForward />}
                              sx={{
                                px: 4,
                                py: 1.5,
                                fontWeight: 600,
                                borderRadius: 2,
                              }}
                            >
                              {product.pricing_options_count > 0 ? 'View Pricing Options' : 'Learn More'}
                            </Button>
                          </Box>
                        </CardContent>
                      </Grid>
                    </Grid>
                  </Card>
                </Grow>
              </Grid>
            ))}
          </Grid>
        )}
      </Container>
    </>
  );

  return (
    <>
      <JsonLd data={breadcrumbSchema([
        { name: 'Home', url: '/' },
        { name: 'Products' },
      ])} />

      {selectedCategory ? <ProductsInCategoryView /> : <CategoriesView />}
    </>
  );
};

export default ProductsContent;
