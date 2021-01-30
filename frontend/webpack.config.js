const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
  entry: "./src/bootstrap.ts",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "index.js",
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        loader: 'ts-loader',
        exclude: /node_modules/,
      },
      {
        test: /\.rle$/,
        use: 'raw-loader',
      }
    ]
  },
  resolve: {
    extensions: ['.ts', '.js'],
  },
  mode: "development",
  plugins: [
    new CopyWebpackPlugin(['static/index.html'])
  ],
};
