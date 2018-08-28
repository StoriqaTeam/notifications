UPDATE templates 
SET data = '<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">
<html lang="en">

<head>
    <title>Password reset</title>
    <meta http-equiv="Content-Type" content="text/html charset=UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta http-equiv="X-UA-Compatible" content="IE=edge" />
    <style type="text/css">
        @font-face {
            font-family: "Circe";
            src: url("https://s3.eu-central-1.amazonaws.com/dumpster.stq/fonts/Circe/Circe-Regular.eot");
            src: local("Circe"), local("Circe-Regular"), url("https://s3.eu-central-1.amazonaws.com/dumpster.stq/fonts/Circe/Circe-Regular.eot?#iefix") format("embedded-opentype"), url("https://s3.eu-central-1.amazonaws.com/dumpster.stq/fonts/Circe/Circe-Regular.woff") format("woff"), url("https://s3.eu-central-1.amazonaws.com/dumpster.stq/fonts/Circe/Circe-Regular.ttf") format("truetype");
            font-weight: normal;
            font-style: normal;
        }

        @font-face {
            font-family: "Circe Bold";
            src: url("https://s3.eu-central-1.amazonaws.com/dumpster.stq/fonts/Circe/Circe-Bold.eot");
            src: local("Circe Bold"), local("Circe-Bold"), url("https://s3.eu-central-1.amazonaws.com/dumpster.stq/fonts/Circe/Circe-Bold.eot?#iefix") format("embedded-opentype"), url("https://s3.eu-central-1.amazonaws.com/dumpster.stq/fonts/Circe/Circe-Bold.woff") format("woff"), url("https://s3.eu-central-1.amazonaws.com/dumpster.stq/fonts/Circe/Circe-Bold.ttf") format("truetype");
            font-weight: bold;
            font-style: normal;
        }

        body {
            font-family: Circe, Helvetica, Arial, sans-serif;
            width: 100% !important;
            min-width: 100%;
            -webkit-text-size-adjust: 100%;
            -ms-text-size-adjust: 100%;
            margin: 0;
            Margin: 0;
            padding: 0;
        }

        th,
        td {
            word-wrap: break-word;
            hyphens: auto;
        }

        h1 {
            margin: 0;
            mso-line-height-rule: exactly;
        }

        p {
            margin: 0;
        }

        img {
            outline: none;
            text-decoration: none;
            -ms-interpolation-mode: bicubic;
            width: auto;
            max-width: 100%;
            clear: both;
            display: block;
        }

        strong {
            font-family: Circe Bold, Helvetica, Arial, sans-serif;
        }

        table.body {
            background: #eaeced;
            height: 100%;
            width: 100%;
        }

        table.header {
            width: 100%;
            padding: 0 20px;
        }

        table.header-wrap {
            width: 100%;
            max-width: 600px;
        }

        table.header-wrap th.logo {
            width: 100%;
            padding: 30px 0 40px;
        }

        table.header-wrap th.logo img {
            width: 178px;
        }

        table.container {
            width: 100%;
            padding: 0 20px;
        }

        table.container-wrap {
            width: 100%;
            max-width: 600px;
            min-height: 580px;
            background: #ffffff;
            padding: 65px 80px 65px;
        }

        table.container-wrap td.top {
            vertical-align: top;
        }

        table.container-wrap td.bottom {
            vertical-align: bottom;
        }

        table.container-wrap td.bottom p.postscript {
            color: #888888;
            font-size: 14px;
            line-height: 24px;
            font-style: italic;
        }

        table.container-wrap h1.title {
            font-size: 34px;
            color: #292c34;
            margin-bottom: 60px;
            line-height: 40px;
        }

        table.container-wrap p.desc {
            font-size: 16px;
            line-height: 24px;
            color: #888888;
            margin-bottom: 50px;
        }

        table.container-wrap p.button {
            text-align: center;
            line-height: 64px;
            margin-bottom: 120px;
        }

        table.container-wrap p.button a.button-link {
            text-decoration: none;
            color: #ffffff;
            background: #03a9ff;
            padding: 22px 55px 18px;
        }

        table.container-wrap p.regards {
            font-size: 16px;
            line-height: 24px;
            color: #292c34;
            margin-bottom: 40px;
        }

        table.container-wrap p.note {
            color: #292c34;
            font-size: 14px;
            line-height: 24px;
        }

        table.footer {
            width: 100%;
            padding: 0 20px;
        }

        table.footer-wrap {
            width: 100%;
            max-width: 600px;
        }

        table.footer-wrap p.postscript {
            margin-top: -90px;
            padding-left: 80px;
            color: #888888;
            font-size: 14px;
            line-height: 24px;
            font-style: italic;
        }

        table.footer-wrap td.social-icon {
            padding: 10px;
        }

        table.footer-wrap td.social-icon img {
            width: 16px;
            height: 16px;
        }

        table.footer-wrap td.social-icon a {
            text-transform: none;
        }

        table.social {
            margin-top: 30px;
            margin-bottom: 15px;
        }

        table.footer-info td {
            text-align: center;
        }

        table.footer-info p.footer-text-a {
            margin-top: 32px;
            color: #292c34;
            font-size: 15px;
        }

        table.footer-info p.footer-text-b {
            max-width: 260px;
            margin: 20px auto 0;
            font-size: 13px;
            color: #888888;
        }

        table.footer-info p.footer-text-c {
            margin-top: 25px;
            margin-bottom: 50px;
            font-size: 13px;
            color: #888888;
        }

        table.footer-info a.footer-text-link {
            font-size: 15px;
            color: #292c34;
            font-weight: bold;
        }

        table.footer-info a.footer-text-link:visited {
            color: #292c34;
        }

        @media only screen and (max-width: 596px) {
            table.container-wrap {
                padding: 30px 40px;
            }
        }
    </style>
</head>

<body>
    <table bgcolor="#eaeced" class="body">
        <tr>
            <td class="float-center" align="center" valign="top">
                <table align="center" class="header float-center">
                    <tr>
                        <td class="">
                            <table align="center" class="header-wrap">
                                <tr>
                                    <th class="logo"> <img src="https://s3.eu-central-1.amazonaws.com/dumpster.stq/img/storiqa-logo.png"> </th>
                                </tr>
                            </table>
                        </td>
                    </tr>
                </table>
                <table align="center" class="container float-center">
                    <tbody>
                        <tr>
                            <td>
                                <table align="center" class="container-wrap">
                                    <tbody>
                                        <tr>
                                            <td class="top">
                                                <h1 class="title">Password reset</h1>
                                                <p class="desc">Dear {{user.first_name}}! You received this e-mail because you have made
                                                    a request to change your password. In order to do that, please follow
                                                    the link below:</p>
                                                <p class="button"> <a href="{{reset_password_path}}/{{token}}" class="button-link" target="_blank">RESET
                                                        PASSWORD</a> </p>
                                                <p class="regards">Best regards,<br>Storiqa Team</p>
                                                <p class="note"><strong>Note</strong>: If you have received a password-reset email you didn&apos;t
                                                    request, it&apos;s likely that someone entered your e-mail address by
                                                    mistake. If you didn&apos;t initiate this request, please delete this
                                                    e-mail. Your privacy and security aren&apos;t compromised by this e-mail.</p>
                                            </td>
                                        </tr>
                                        <tr>
                                            <td class="bottom"><p class="postscript">This is an automatically generated e-mail – please do not reply to it.</p></td>
                                        </tr>
                                    </tbody>
                                </table>
                            </td>
                        </tr>
                    </tbody>
                </table>
                <table align="center" class="footer float-center">
                    <tbody>
                        <tr>
                            <td>
                                <table align="center" class="footer-wrap">
                                    <tbody>
                                        <tr>
                                            <td>
                                                <table align="center" class="social">
                                                    <tbody>
                                                        <tr>
                                                            <td class="social-icon"> <a href="https://www.facebook.com/storiqa" target="_blank"> <img
                                                                        src="https://s3.eu-central-1.amazonaws.com/dumpster.stq/img/facebook-logo.png">                                                                    </a>
                                                                </td>
                                                            <td class="social-icon"> <a href="https://twitter.com/storiqa" target="_blank"> <img src="https://s3.eu-central-1.amazonaws.com/dumpster.stq/img/twitter-logo.png">                                                                    </a>
                                                                </td>
                                                            <td class="social-icon"> <a href="https://www.linkedin.com/showcase/24773372" target="_blank">
                                                                <img src="https://s3.eu-central-1.amazonaws.com/dumpster.stq/img/linkedin-logo.png">                                                                    </a>
                                                                </td>
                                                        </tr>
                                                    </tbody>
                                                </table>
                                            </td>
                                        </tr>
                                        <tr>
                                            <td>
                                                <table align="center" class="footer-info">
                                                    <tbody>
                                                        <tr>
                                                            <td>
                                                                <p class="footer-text-a">For all questions send email to <a href="mailto:info@storiqa.com"
                                                                        class="footer-text-link" target="_blank">info@storiqa.com</a></p>
                                                                <p
                                                                    class="footer-text-b">You have received this letter because you had registered
                                                                    in Storica.com.</p>
                                                                <p class="footer-text-c">Storiqa, 2018. All Rights Reserved.</p>
                                                            </td>
                                                        </tr>
                                                    </tbody>
                                                </table>
                                            </td>
                                        </tr>
                                    </tbody>
                                </table>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </td>
        </tr>
    </table>
</body>

</html>'
WHERE
name = 'password_reset_for_user';