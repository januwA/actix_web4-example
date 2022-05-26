use crate::prelude::*;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpMessage};
use futures_util::future::LocalBoxFuture;

use std::future::{ready, Ready};

/// jwt 验证
///
pub struct JWTAuth;
impl<S, B> Transform<S, ServiceRequest> for JWTAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JWTAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JWTAuthMiddleware { service }))
    }
}

pub struct JWTAuthMiddleware<S> {
    service: S,
}

impl<S, B> JWTAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    fn get_token<'a>(&self, req: &'a ServiceRequest) -> Result<&'a str, ()> {
        Ok(req
            .headers()
            .get("Authorization")
            .ok_or(())?
            .to_str()
            .map_err(|_| ())?)
    }
}

impl<S, B> Service<ServiceRequest> for JWTAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if let Ok(authorization) = self.get_token(&req) {
            // 7.. => 'Bearer ' + token
            match Claims::decode(&authorization[7..]) {
                Ok(token_data) => {
                    req.extensions_mut().insert(token_data.claims);
                }
                Err(err) => {
                    return Box::pin(async move { Err(ej(&err).auth().actix()) });
                }
            }
        } else {
            return Box::pin(async { Err(ej("需要身份验证").auth().actix()) });
        }

        let fut = self.service.call(req);

        Box::pin(async move { Ok(fut.await?) })
    }
}

/// 请求实体大小
///
pub struct ContentLengthLimit {
    pub limit: u64, // byte
}
impl<S, B> Transform<S, ServiceRequest> for ContentLengthLimit
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ContentLengthLimitMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ContentLengthLimitMiddleware {
            service,
            limit: self.limit,
        }))
    }
}

impl Default for ContentLengthLimit {
    fn default() -> Self {
        Self {
            limit: 1024 * 1024 * 5, /* 5M */
        }
    }
}

pub struct ContentLengthLimitMiddleware<S> {
    service: S,
    limit: u64, // byte
}

impl<S, B> ContentLengthLimitMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    fn is_big(&self, req: &ServiceRequest) -> Result<bool, ()> {
        req.headers()
            .get("content-length")
            .ok_or(())? // Option可以使用 ok_or 来映射为 None的情况
            .to_str()
            .map_err(|_| ())? // map_err 可以处理 Result 返回 Err的情况
            .parse::<u64>()
            .and_then(|cl| Ok(cl > self.limit))
            .map_err(|_| ())
    }
}

impl<S, B> Service<ServiceRequest> for ContentLengthLimitMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if let Ok(r) = self.is_big(&req) {
            if r {
                return Box::pin(async { res_err!("请求实体太大") });
            }
        }

        let fut = self.service.call(req);
        Box::pin(async move { Ok(fut.await?) })
    }
}

/// 自定义Cors跨域头设置
///
/// ```
/// App::new().wrap(middleware::HttpCors)
/// ```
///
/// https://support.huaweicloud.com/usermanual-cdn/cdn_01_0021.html
/// https://developer.mozilla.org/zh-CN/docs/Web/HTTP/CORS
/// https://enable-cors.org/server_nginx.html
pub struct HttpCors;
impl<S, B> Transform<S, ServiceRequest> for HttpCors
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = HttpCorsMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(HttpCorsMiddleware { service }))
    }
}

pub struct HttpCorsMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for HttpCorsMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        use actix_web::http::header::{
            HeaderValue, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN,
            ACCESS_CONTROL_MAX_AGE,
        };

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;
            let headers = res.headers_mut();
            headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
            headers.insert(
                ACCESS_CONTROL_ALLOW_METHODS,
                HeaderValue::from_static("OPTION,GET,POST,DELETE,PUT,PATCH"),
            );
            headers.insert(ACCESS_CONTROL_MAX_AGE, HeaderValue::from_static("1728000"));
            Ok(res)
        })
    }
}
