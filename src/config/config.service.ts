import { config } from "dotenv";

export class ConfigService {
  public readonly host: string;
  public readonly port: number;
  public readonly reverse_proxy_enabled: boolean;

  constructor() {
    const parsed = config().parsed;

    if (!parsed) {
      throw new Error("Invalid .env file");
    }

    this.host = process.env.HOST || "localhost";
    this.port = Number(process.env.PORT) || 8080;
    this.reverse_proxy_enabled =
      Boolean(process.env.REVERSE_PROXY_ENABLED) || false;
  }
}
