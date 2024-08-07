import { Catch, HttpException, HttpStatus, type ArgumentsHost, ExceptionFilter } from "@nestjs/common";
import type { FastifyRequest } from "fastify";
import type { FastifyReply } from "fastify/types/reply";
import { ResponseModel } from "../models/response.model";

@Catch()
export class AllExceptionsFilter implements ExceptionFilter {
    catch(exception: unknown, host: ArgumentsHost) {
        console.log('🚀 ~ AllExceptionsFilter ~ exception:', exception);
        const ctx = host.switchToHttp();
        const response = ctx.getResponse<FastifyReply>();
        const request = ctx.getRequest<FastifyRequest>();
        const status = 
        exception instanceof HttpException
            ? exception.getStatus()
            : HttpStatus.INTERNAL_SERVER_ERROR;
        
        response.status(status).send(ResponseModel.error(status, '服务器内部错误', request.url, 'INTERNAL_SERVER_ERROR'))
    }
}