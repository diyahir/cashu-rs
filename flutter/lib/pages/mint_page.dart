// ignore_for_file: use_build_context_synchronously

import 'package:moksha_wallet/ffi.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:moksha_wallet/pages/util.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:share_plus/share_plus.dart';

class MintPage extends StatelessWidget {
  const MintPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const Center(
      child: Column(
        children: [MintWidget()],
      ),
    );
  }
}

class MintWidget extends StatefulWidget {
  const MintWidget({super.key});

  @override
  State<MintWidget> createState() => _MintWidgetState();
}

class _MintWidgetState extends State<MintWidget> {
  bool _isInvoiceCreated = false;
  String amount = '';
  FlutterPaymentRequest? paymentRequest;
  final _textController = TextEditingController();

  @override
  void dispose() {
    super.dispose();
    _textController.dispose();
  }

  @override
  void initState() {
    super.initState();

    _textController.addListener(() {
      final regEx = RegExp(r'(\d{1,3})(?=(\d{3})+(?!\d))');
      matchFunc(Match match) => '${match[1]},';
      final text = _textController.text;
      _textController.value = _textController.value.copyWith(
        text: text.replaceAll(',', '').replaceAllMapped(regEx, matchFunc),
        selection: TextSelection(
          baseOffset: text.length,
          extentOffset: text.length,
        ),
      );
    });
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.all(24),
      padding: const EdgeInsets.all(24),
      child: Column(
        children: [
          _isInvoiceCreated
              ? Column(
                  children: [
                    QrImageView(
                      data: paymentRequest!.pr,
                      version: QrVersions.auto,
                      size: 200.0,
                      backgroundColor: Colors.white,
                      eyeStyle: const QrEyeStyle(
                        eyeShape: QrEyeShape.square,
                        color: Colors.black,
                      ),
                    ),
                    Container(
                      padding: const EdgeInsets.all(8),
                      margin: const EdgeInsets.all(8),
                      child: Row(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          const Spacer(),
                          ElevatedButton(
                              onPressed: () {
                                Clipboard.setData(
                                  ClipboardData(
                                    text: paymentRequest!.pr,
                                  ),
                                );

                                if (!context.mounted) return;
                                ScaffoldMessenger.of(context)
                                    .showSnackBar(const SnackBar(
                                  content: Column(children: [
                                    Text('Copied invoice to clipboard'),
                                  ]),
                                  showCloseIcon: true,
                                ));
                              },
                              child: const Text('Copy')),
                          const SizedBox(width: 8),
                          ElevatedButton(
                              onPressed: () {
                                Share.share(paymentRequest!.pr);
                              },
                              child: const Text('Share')),
                          const Spacer(),
                        ],
                      ),
                    )
                  ],
                )
              : const Text(
                  'Pay a lighting invoice to mint cashu tokens',
                  style: TextStyle(fontSize: 20),
                ),
          Container(
            margin: const EdgeInsets.all(24),
            padding: const EdgeInsets.all(24),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
              children: [
                const Spacer(),
                Visibility(
                    visible: !_isInvoiceCreated,
                    child: Flexible(
                      fit: FlexFit.loose,
                      child: Padding(
                        padding: const EdgeInsets.all(8),
                        child: TextField(
                          controller: _textController,
                          inputFormatters: [
                            LengthLimitingTextInputFormatter(9),
                            FilteringTextInputFormatter.digitsOnly,
                          ],
                          keyboardType: TextInputType.number,
                          decoration: const InputDecoration(
                            border: OutlineInputBorder(),
                            labelText: 'Amount (sats)',
                          ),
                          onChanged: (value) => setState(() {
                            amount = value;
                          }),
                        ),
                      ),
                    )),
                Visibility(
                    visible: !_isInvoiceCreated,
                    child: ElevatedButton(
                        onPressed: () async {
                          if (amount == '' || amount == '0') {
                            if (!context.mounted) return;
                            ScaffoldMessenger.of(context)
                                .showSnackBar(const SnackBar(
                              content: Column(children: [
                                Text('Amount must be greater than 0')
                              ]),
                              showCloseIcon: true,
                            ));
                            return;
                          }

                          var cleanAmount =
                              int.parse(amount.replaceAll(",", ""));

                          try {
                            var result = await api.getMintPaymentRequest(
                                amount: cleanAmount); // use decimalTextfield
                            setState(() {
                              paymentRequest = result;
                              _isInvoiceCreated = true;
                            });
                          } catch (e) {
                            if (!context.mounted) return;
                            showErrorSnackBar(
                                context, e, "Error creating invoice");
                            return;
                          }

                          try {
                            var mintedTokens = await api.mintTokens(
                                amount: cleanAmount,
                                hash: paymentRequest!.hash);
                            setState(() {
                              paymentRequest = null;
                              _isInvoiceCreated = false;
                              amount = ''; // FIMXE clear textfield
                            });

                            if (!context.mounted) return;
                            ScaffoldMessenger.of(context).showSnackBar(SnackBar(
                              content: Column(children: [
                                Text('Minted $mintedTokens sats')
                              ]),
                              showCloseIcon: true,
                            ));
                          } catch (e) {
                            if (!context.mounted) return;
                            showErrorSnackBar(
                                context, e, "Error creating invoice");
                            return;
                          }
                        },
                        child: const Text('Mint tokens'))),
                const Spacer(),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
